//! Grafo de cena: entidades, hierarquia e selecao.
//!
//! Deliberadamente *nao* e um ECS. Um ECS de verdade e um projeto por si so, e
//! a decisao provavel e adotar `bevy_ecs` em vez de escrever o nosso. Isto aqui
//! e o minimo que o editor precisa para ter algo real para listar e editar — e
//! e pequeno o bastante para ser jogado fora quando o ECS entrar.

use rugine_core::math::{Quat, Vec3};

pub mod settings;

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
#[derive(Clone, Debug)]
pub struct Entity {
    pub name: String,
    pub transform: Transform,
    pub visible: bool,
    parent: Option<EntityId>,
    children: Vec<EntityId>,
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
#[derive(Clone, Debug)]
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
}
