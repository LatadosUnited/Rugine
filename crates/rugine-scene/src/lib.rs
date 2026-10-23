//! Grafo de cena: entidades, hierarquia e selecao.
//!
//! Deliberadamente *nao* e um ECS. Um ECS de verdade e um projeto por si so, e
//! a decisao provavel e adotar `bevy_ecs` em vez de escrever o nosso. Isto aqui
//! e o minimo que o editor precisa para ter algo real para listar e editar — e
//! e pequeno o bastante para ser jogado fora quando o ECS entrar.

use std::any::Any;

use rugine_core::math::{Quat, Vec3};

pub mod componentes;
pub mod settings;

pub use componentes::{Colisor, Componente, FormaColisor, Malha};
pub use settings::SceneSettings;

/// Identificador de uma entidade.
///
/// Carrega uma geracao junto do indice: sem ela, remover uma entidade e criar
/// outra reaproveitaria o slot, e um id antigo passaria a apontar para a
/// entidade errada em silencio — o classico "use after free" de arenas.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityId {
    index: u32,
    generation: u32,
}

/// Posicao, rotacao e escala de uma entidade, relativas ao pai.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl Transform {
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            ..Default::default()
        }
    }

    /// Matriz local desta entidade (ainda nao composta com a do pai).
    pub fn to_matrix(self) -> rugine_core::math::Mat4 {
        rugine_core::math::Mat4::from_scale_rotation_translation(
            self.scale,
            self.rotation,
            self.translation,
        )
    }
}

/// Um no da cena.
///
/// Sem `Clone`: componentes sao objetos de tipo apagado, e duplicar uma
/// entidade vai pedir um `clonar` em cada um — decisao para quando houver
/// "Duplicar" no editor.
#[derive(Debug)]
pub struct Entity {
    pub name: String,
    pub transform: Transform,
    pub visible: bool,
    parent: Option<EntityId>,
    children: Vec<EntityId>,
    /// Privado de proposito: toda mudanca passa pelos metodos da `Scene`.
    /// E o funil por onde o desfazer/refazer vai entrar depois.
    componentes: Vec<Box<dyn Componente>>,
}

impl Entity {
    pub fn parent(&self) -> Option<EntityId> {
        self.parent
    }

    pub fn children(&self) -> &[EntityId] {
        &self.children
    }
}

/// Slot da arena: ou tem entidade, ou esta livre aguardando reuso.
#[derive(Debug)]
struct Slot {
    generation: u32,
    entity: Option<Entity>,
}

/// A cena.
#[derive(Default)]
pub struct Scene {
    slots: Vec<Slot>,
    /// Slots livres, para reusar indices em vez de crescer sem parar.
    free: Vec<u32>,
    roots: Vec<EntityId>,
    selected: Option<EntityId>,
    pub settings: SceneSettings,
}

impl Scene {
    pub fn new() -> Self {
        Self::default()
    }

    /// Cria uma entidade. `parent` `None` a coloca na raiz.
    pub fn spawn(&mut self, name: impl Into<String>, parent: Option<EntityId>) -> EntityId {
        let entity = Entity {
            name: name.into(),
            transform: Transform::default(),
            visible: true,
            parent,
            children: Vec::new(),
            componentes: Vec::new(),
        };

        let id = match self.free.pop() {
            Some(index) => {
                let slot = &mut self.slots[index as usize];
                // Avanca a geracao: ids antigos que apontavam para este slot
                // passam a nao resolver mais.
                slot.generation += 1;
                slot.entity = Some(entity);
                EntityId {
                    index,
                    generation: slot.generation,
                }
            }
            None => {
                self.slots.push(Slot {
                    generation: 0,
                    entity: Some(entity),
                });
                EntityId {
                    index: (self.slots.len() - 1) as u32,
                    generation: 0,
                }
            }
        };

        match parent {
            Some(parent_id) => {
                if let Some(parent) = self.get_mut(parent_id) {
                    parent.children.push(id);
                } else {
                    log::warn!("spawn com pai invalido; a entidade foi para a raiz");
                    self.roots.push(id);
                }
            }
            None => self.roots.push(id),
        }

        id
    }

    /// Remove uma entidade e toda a sua descendencia.
    pub fn despawn(&mut self, id: EntityId) {
        if !self.contains(id) {
            return;
        }

        // Desliga do pai antes de destruir, senao o pai fica com um filho que
        // nao existe mais.
        let parent = self.get(id).and_then(|e| e.parent);
        match parent {
            Some(parent_id) => {
                if let Some(parent) = self.get_mut(parent_id) {
                    parent.children.retain(|child| *child != id);
                }
            }
            None => self.roots.retain(|root| *root != id),
        }

        self.despawn_recursive(id);

        // A selecao pode ter apontado para algo que acabou de sumir.
        if let Some(selected) = self.selected
            && !self.contains(selected)
        {
            self.selected = None;
        }
    }

    /// Destroi a subarvore sem mexer no pai (ja desligado por `despawn`).
    fn despawn_recursive(&mut self, id: EntityId) {
        let children = match self.get(id) {
            Some(entity) => entity.children.clone(),
            None => return,
        };

        for child in children {
            self.despawn_recursive(child);
        }

        if let Some(slot) = self.slots.get_mut(id.index as usize) {
            slot.entity = None;
            self.free.push(id.index);
        }
    }

    /// `true` se o id ainda aponta para uma entidade viva.
    pub fn contains(&self, id: EntityId) -> bool {
        self.slot(id).is_some()
    }

    fn slot(&self, id: EntityId) -> Option<&Slot> {
        self.slots
            .get(id.index as usize)
            .filter(|slot| slot.generation == id.generation && slot.entity.is_some())
    }

    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.slot(id).and_then(|slot| slot.entity.as_ref())
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.slots
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.entity.as_mut())
    }

    /// Entidades sem pai, na ordem de criacao.
    pub fn roots(&self) -> &[EntityId] {
        &self.roots
    }

    /// Toda entidade viva, em ordem de slot.
    ///
    /// Ordem de slot, nao de hierarquia: quem desenha nao se importa com
    /// parentesco (a matriz de mundo ja o resolve), e percorrer a arvore
    /// custaria recursao por frame sem ganho nenhum.
    pub fn entidades(&self) -> impl Iterator<Item = EntityId> + '_ {
        self.slots
            .iter()
            .enumerate()
            .filter(|(_, slot)| slot.entity.is_some())
            .map(|(indice, slot)| EntityId {
                index: indice as u32,
                generation: slot.generation,
            })
    }

    /// Quantas entidades vivas existem.
    pub fn len(&self) -> usize {
        self.slots.iter().filter(|s| s.entity.is_some()).count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn selected(&self) -> Option<EntityId> {
        self.selected
    }

    pub fn select(&mut self, id: Option<EntityId>) {
        self.selected = match id {
            Some(id) if self.contains(id) => Some(id),
            Some(_) => None,
            None => None,
        };
    }

    /// Pendura um componente na entidade.
    ///
    /// Devolve `false`, sem mudar nada, se a entidade nao existe ou ja tem um
    /// componente do mesmo tipo.
    pub fn adicionar<C: Componente>(&mut self, id: EntityId, componente: C) -> bool {
        self.adicionar_box(id, Box::new(componente))
    }

    /// Como [`Self::adicionar`], para quando o tipo so e conhecido em tempo de
    /// execucao — o menu "Adicionar componente" do editor.
    pub fn adicionar_box(&mut self, id: EntityId, componente: Box<dyn Componente>) -> bool {
        let Some(entidade) = self.get_mut(id) else {
            return false;
        };

        // `type_id` pelo `dyn Any`, e nao pelo `Box`: no `Box` ele devolveria
        // o tipo do proprio `Box`, igual para todos os componentes.
        let tipo = (componente.as_ref() as &dyn Any).type_id();
        if entidade
            .componentes
            .iter()
            .any(|c| (c.as_ref() as &dyn Any).type_id() == tipo)
        {
            return false;
        }

        entidade.componentes.push(componente);
        true
    }

    /// Tira o componente deste tipo e o devolve.
    pub fn remover<C: Componente>(&mut self, id: EntityId) -> Option<C> {
        let entidade = self.get_mut(id)?;
        let indice = entidade
            .componentes
            .iter()
            .position(|c| (c.as_ref() as &dyn Any).is::<C>())?;

        let removido: Box<dyn Any> = entidade.componentes.remove(indice);
        removido.downcast::<C>().ok().map(|c| *c)
    }

    /// Tira o componente desta posicao da lista, sem conhecer o tipo.
    pub fn remover_indice(&mut self, id: EntityId, indice: usize) -> Option<Box<dyn Componente>> {
        let entidade = self.get_mut(id)?;
        (indice < entidade.componentes.len()).then(|| entidade.componentes.remove(indice))
    }

    pub fn componente<C: Componente>(&self, id: EntityId) -> Option<&C> {
        self.get(id)?
            .componentes
            .iter()
            .find_map(|c| (c.as_ref() as &dyn Any).downcast_ref::<C>())
    }

    pub fn componente_mut<C: Componente>(&mut self, id: EntityId) -> Option<&mut C> {
        self.get_mut(id)?
            .componentes
            .iter_mut()
            .find_map(|c| (c.as_mut() as &mut dyn Any).downcast_mut::<C>())
    }

    pub fn tem<C: Componente>(&self, id: EntityId) -> bool {
        self.componente::<C>(id).is_some()
    }

    /// Os componentes da entidade, na ordem em que foram adicionados.
    ///
    /// Vazio para um id morto: quem lista componentes nao precisa tratar esse
    /// caso a parte.
    pub fn componentes(&self, id: EntityId) -> &[Box<dyn Componente>] {
        self.get(id).map_or(&[], |e| &e.componentes)
    }

    pub fn componentes_mut(&mut self, id: EntityId) -> &mut [Box<dyn Componente>] {
        match self.get_mut(id) {
            Some(entidade) => &mut entidade.componentes,
            None => &mut [],
        }
    }

    /// Toda entidade viva com um componente deste tipo.
    pub fn com<C: Componente>(&self) -> impl Iterator<Item = (EntityId, &C)> + '_ {
        self.entidades()
            .filter_map(|id| self.componente::<C>(id).map(|c| (id, c)))
    }

    /// Matriz de mundo, compondo a cadeia de pais ate a raiz.
    pub fn world_matrix(&self, id: EntityId) -> rugine_core::math::Mat4 {
        let mut matrix = rugine_core::math::Mat4::IDENTITY;
        let mut current = Some(id);

        // Sobe ate a raiz acumulando; para uma cena de editor isto e barato.
        // Com milhares de entidades por frame, viraria um cache de transforms.
        while let Some(entity_id) = current {
            let Some(entity) = self.get(entity_id) else {
                break;
            };
            matrix = entity.transform.to_matrix() * matrix;
            current = entity.parent;
        }

        matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn despawn_leva_a_descendencia_junto() {
        let mut scene = Scene::new();
        let pai = scene.spawn("Pai", None);
        let filho = scene.spawn("Filho", Some(pai));
        let neto = scene.spawn("Neto", Some(filho));

        assert_eq!(scene.len(), 3);
        scene.despawn(pai);

        assert_eq!(scene.len(), 0, "a subarvore inteira deveria ter sumido");
        assert!(!scene.contains(filho));
        assert!(!scene.contains(neto));
        assert!(scene.roots().is_empty());
    }

    /// O motivo de `EntityId` carregar geracao.
    #[test]
    fn id_antigo_nao_resolve_para_entidade_nova() {
        let mut scene = Scene::new();
        let antigo = scene.spawn("Antiga", None);
        scene.despawn(antigo);

        let novo = scene.spawn("Nova", None);

        assert!(!scene.contains(antigo), "o id antigo deveria estar morto");
        assert!(scene.contains(novo));
        assert_eq!(
            antigo.index, novo.index,
            "o teste so vale se o slot foi reusado"
        );
    }

    #[test]
    fn selecao_e_limpa_quando_o_alvo_morre() {
        let mut scene = Scene::new();
        let id = scene.spawn("Alvo", None);
        scene.select(Some(id));
        assert_eq!(scene.selected(), Some(id));

        scene.despawn(id);
        assert_eq!(scene.selected(), None);
    }

    #[test]
    fn world_matrix_compoe_a_cadeia_de_pais() {
        let mut scene = Scene::new();
        let pai = scene.spawn("Pai", None);
        let filho = scene.spawn("Filho", Some(pai));

        scene.get_mut(pai).unwrap().transform.translation = Vec3::new(10.0, 0.0, 0.0);
        scene.get_mut(filho).unwrap().transform.translation = Vec3::new(0.0, 5.0, 0.0);

        let mundo = scene.world_matrix(filho);
        let posicao = mundo.transform_point3(Vec3::ZERO);

        assert!(
            (posicao - Vec3::new(10.0, 5.0, 0.0)).length() < 1e-5,
            "deu {posicao}"
        );
    }

    #[test]
    fn adicionar_e_ler_componente() {
        let mut scene = Scene::new();
        let id = scene.spawn("Caixa", None);

        assert!(!scene.tem::<Malha>(id));
        assert!(scene.adicionar(id, Malha::default()));
        assert!(scene.tem::<Malha>(id));
        assert!(!scene.tem::<Colisor>(id), "so a Malha foi adicionada");

        scene.componente_mut::<Malha>(id).unwrap().cor = settings::Color::new(1.0, 0.0, 0.0);
        assert_eq!(scene.componente::<Malha>(id).unwrap().cor.r, 1.0);
    }

    /// Um de cada tipo por entidade: o segundo e recusado e o primeiro fica.
    #[test]
    fn nao_aceita_dois_do_mesmo_tipo() {
        let mut scene = Scene::new();
        let id = scene.spawn("Caixa", None);
        let vermelha = Malha {
            cor: settings::Color::new(1.0, 0.0, 0.0),
        };

        assert!(scene.adicionar(id, vermelha));
        assert!(!scene.adicionar(id, Malha::default()));
        assert!(
            scene.adicionar(id, Colisor::default()),
            "tipo diferente entra"
        );

        assert_eq!(scene.componentes(id).len(), 2);
        assert_eq!(scene.componente::<Malha>(id), Some(&vermelha));
    }

    /// A ordem de adicao e a ordem do Inspector, e remover por indice tira o
    /// componente certo sem que o chamador conheca o tipo.
    #[test]
    fn ordem_e_remocao() {
        let mut scene = Scene::new();
        let id = scene.spawn("Caixa", None);
        scene.adicionar(id, Colisor::default());
        scene.adicionar(id, Malha::default());

        let nomes: Vec<_> = scene.componentes(id).iter().map(|c| c.nome()).collect();
        assert_eq!(nomes, ["Colisor", "Malha"]);

        let removido = scene.remover_indice(id, 0).unwrap();
        assert_eq!(removido.nome(), "Colisor");
        assert!(scene.remover_indice(id, 5).is_none());

        assert_eq!(scene.remover::<Malha>(id), Some(Malha::default()));
        assert!(scene.componentes(id).is_empty());
    }

    /// Componentes morrem com a entidade, e um id antigo nao enxerga os da
    /// entidade nova que reusou o slot.
    #[test]
    fn componentes_nao_vazam_para_o_slot_reusado() {
        let mut scene = Scene::new();
        let antiga = scene.spawn("Antiga", None);
        scene.adicionar(antiga, Malha::default());
        scene.despawn(antiga);

        let nova = scene.spawn("Nova", None);
        assert!(scene.componentes(nova).is_empty());
        assert!(scene.componentes(antiga).is_empty());
        assert!(
            !scene.adicionar(antiga, Malha::default()),
            "id morto nao aceita"
        );
    }

    #[test]
    fn com_lista_so_quem_tem_o_componente() {
        let mut scene = Scene::new();
        let a = scene.spawn("A", None);
        let _b = scene.spawn("B", None);
        let c = scene.spawn("C", None);
        scene.adicionar(a, Malha::default());
        scene.adicionar(c, Malha::default());

        let ids: Vec<_> = scene.com::<Malha>().map(|(id, _)| id).collect();
        assert_eq!(ids, [a, c]);
    }
}
