//! Salvar e carregar cenas.
//!
//! O arquivo e JSON, e nao RON: cada componente e gravado pelo nome, com os
//! dados num valor generico, e o `ron::Value` nao representa enums — o
//! `FormaColisor` nao sobreviveria a ida e volta. O `serde_json::Value`
//! representa, e o arquivo continua legivel em qualquer ferramenta.
//!
//! O que vai para o arquivo e uma forma propria (`ArquivoCena`), e nao a
//! `Scene` derivada direto: a `Scene` e uma arena com slots livres, geracoes e
//! selecao — detalhes de memoria que nao tem significado fora do processo.
//! No arquivo, a hierarquia e um aninhamento de filhos.

use std::any::{Any, TypeId};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::componentes::{Colisor, Componente, Malha};
use crate::{EntityId, Scene, SceneSettings, Transform};

/// Versao do formato que este codigo escreve, e a maior que ele le.
///
/// Sobe quando um arquivo antigo deixa de ser lido do jeito certo pelo codigo
/// novo. Campo novo com valor padrao nao precisa: o `#[serde(default)]` dos
/// tipos ja cobre a falta dele.
pub const VERSAO: u32 = 1;

/// Extensao dos arquivos de cena.
pub const EXTENSAO: &str = "cena";

#[derive(Debug, thiserror::Error)]
pub enum ErroArquivo {
    #[error("o arquivo nao e uma cena valida: {0}")]
    Formato(#[from] serde_json::Error),
    #[error("cena salva na versao {encontrada} do formato; esta Rugine le ate a {VERSAO}")]
    VersaoNova { encontrada: u32 },
}

/// Os tipos de componente que podem ser salvos, carregados e adicionados.
///
/// Um tipo fora do registro ainda funciona na cena, mas nao vai para o arquivo
/// nem aparece no menu "Adicionar componente". E o mesmo registro que o
/// desfazer vai usar para recriar um componente removido.
pub struct Registro {
    tipos: Vec<TipoRegistrado>,
}

struct TipoRegistrado {
    nome: &'static str,
    tipo: TypeId,
    criar: fn() -> Box<dyn Componente>,
    salvar: fn(&dyn Componente) -> serde_json::Result<Value>,
    carregar: fn(Value) -> serde_json::Result<Box<dyn Componente>>,
}

impl Default for Registro {
    fn default() -> Self {
        Self::padrao()
    }
}

impl Registro {
    pub fn vazio() -> Self {
        Self { tipos: Vec::new() }
    }

    /// Os componentes que a propria Rugine define.
    pub fn padrao() -> Self {
        let mut registro = Self::vazio();
        registro.registrar::<Malha>();
        registro.registrar::<Colisor>();
        registro
    }

    /// Registra um tipo. O nome vem do proprio componente, para nao existir em
    /// dois lugares e divergir.
    pub fn registrar<C>(&mut self)
    where
        C: Componente + Default + Serialize + DeserializeOwned,
    {
        let nome = C::default().nome();
        assert!(
            self.tipo_por_nome(nome).is_none(),
            "dois componentes registrados como {nome:?}: o arquivo nao saberia qual carregar"
        );

        self.tipos.push(TipoRegistrado {
            nome,
            tipo: TypeId::of::<C>(),
            criar: || Box::new(C::default()),
            salvar: salvar_como::<C>,
            carregar: |valor| Ok(Box::new(serde_json::from_value::<C>(valor)?)),
        });
    }

    /// Nomes dos tipos, na ordem de registro.
    pub fn nomes(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.tipos.iter().map(|t| t.nome)
    }

    /// Um componente novo, com os valores padrao, pelo nome.
    pub fn criar(&self, nome: &str) -> Option<Box<dyn Componente>> {
        self.tipo_por_nome(nome).map(|t| (t.criar)())
    }

    fn tipo_por_nome(&self, nome: &str) -> Option<&TipoRegistrado> {
        self.tipos.iter().find(|t| t.nome == nome)
    }

    fn tipo_de(&self, componente: &dyn Componente) -> Option<&TipoRegistrado> {
        let tipo = (componente as &dyn Any).type_id();
        self.tipos.iter().find(|t| t.tipo == tipo)
    }
}

/// Uma funcao nomeada, e nao closure: ela precisa do parametro de tipo `C`, e
/// closures nao tem parametros de tipo proprios.
fn salvar_como<C: Componente + Serialize>(
    componente: &dyn Componente,
) -> serde_json::Result<Value> {
    let concreto = (componente as &dyn Any)
        .downcast_ref::<C>()
        .expect("o registro escolhe a funcao pelo TypeId do proprio componente");
    serde_json::to_value(concreto)
}

#[derive(Serialize, Deserialize)]
struct ArquivoCena {
    versao: u32,
    #[serde(default)]
    configuracoes: SceneSettings,
    #[serde(default)]
    entidades: Vec<EntidadeSalva>,
}

#[derive(Serialize, Deserialize)]
struct EntidadeSalva {
    nome: String,
    #[serde(default = "verdadeiro")]
    visivel: bool,
    #[serde(default)]
    transform: Transform,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    componentes: Vec<ComponenteSalvo>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    filhos: Vec<EntidadeSalva>,
}

fn verdadeiro() -> bool {
    true
}

#[derive(Serialize, Deserialize)]
struct ComponenteSalvo {
    tipo: String,
    dados: Value,
}

/// Uma cena carregada, e o que ficou de fora dela.
pub struct Carregada {
    pub cena: Scene,
    /// Componentes que nao puderam ser recriados: tipo desconhecido, dados
    /// invalidos ou repetidos. A cena abre sem eles em vez de nao abrir.
    pub avisos: Vec<String>,
}

/// A cena como texto, e os componentes que nao puderam ser gravados.
///
/// Fora do registro, um componente nao tem nome no arquivo; ele e pulado e
/// aparece nos avisos, em vez de impedir o resto da cena de ser salvo.
pub fn salvar(cena: &Scene, registro: &Registro) -> (String, Vec<String>) {
    let mut avisos = Vec::new();

    let arquivo = ArquivoCena {
        versao: VERSAO,
        configuracoes: cena.settings,
        entidades: cena
            .roots()
            .iter()
            .filter_map(|&id| entidade_salva(cena, id, registro, &mut avisos))
            .collect(),
    };

    // Nao falha: so mapas com chaves que nao sao texto fazem o `serde_json`
    // recusar, e o formato nao tem nenhum.
    let texto = serde_json::to_string_pretty(&arquivo).expect("o formato so tem chaves de texto");
    (texto, avisos)
}

fn entidade_salva(
    cena: &Scene,
    id: EntityId,
    registro: &Registro,
    avisos: &mut Vec<String>,
) -> Option<EntidadeSalva> {
    let entidade = cena.get(id)?;

    let mut componentes = Vec::new();
    for componente in cena.componentes(id) {
        let componente = componente.as_ref();
        let Some(tipo) = registro.tipo_de(componente) else {
            avisos.push(format!(
                "{}: componente {} nao registrado, nao foi salvo",
                entidade.name,
                componente.nome()
            ));
            continue;
        };

        match (tipo.salvar)(componente) {
            Ok(dados) => componentes.push(ComponenteSalvo {
                tipo: tipo.nome.to_owned(),
                dados,
            }),
            Err(erro) => avisos.push(format!(
                "{}: {} nao foi salvo: {erro}",
                entidade.name, tipo.nome
            )),
        }
    }

    Some(EntidadeSalva {
        nome: entidade.name.clone(),
        visivel: entidade.visible,
        transform: entidade.transform,
        componentes,
        filhos: entidade
            .children()
            .iter()
            .filter_map(|&filho| entidade_salva(cena, filho, registro, avisos))
            .collect(),
    })
}

/// Le uma cena de texto.
///
/// So falha se o arquivo inteiro nao puder ser lido. Um componente com
/// problema vira aviso.
pub fn carregar(texto: &str, registro: &Registro) -> Result<Carregada, ErroArquivo> {
    // A versao primeiro e sozinha: um arquivo de versao nova pode ter uma
    // forma que o resto da leitura nao entende, e o erro certo e "versao
    // nova", nao "campo inesperado".
    #[derive(Deserialize)]
    struct SoVersao {
        versao: u32,
    }
    let SoVersao { versao } = serde_json::from_str(texto)?;
    if versao > VERSAO {
        return Err(ErroArquivo::VersaoNova { encontrada: versao });
    }

    let arquivo: ArquivoCena = serde_json::from_str(texto)?;

    let mut cena = Scene::new();
    cena.settings = arquivo.configuracoes;
    let mut avisos = Vec::new();

    for entidade in arquivo.entidades {
        recriar(&mut cena, entidade, None, registro, &mut avisos);
    }

    Ok(Carregada { cena, avisos })
}

fn recriar(
    cena: &mut Scene,
    salva: EntidadeSalva,
    pai: Option<EntityId>,
    registro: &Registro,
    avisos: &mut Vec<String>,
) {
    let id = cena.spawn(salva.nome.clone(), pai);
    if let Some(entidade) = cena.get_mut(id) {
        entidade.transform = salva.transform;
        entidade.visible = salva.visivel;
    }

    for componente in salva.componentes {
        let Some(tipo) = registro.tipo_por_nome(&componente.tipo) else {
            avisos.push(format!(
                "{}: componente desconhecido {:?}, ignorado",
                salva.nome, componente.tipo
            ));
            continue;
        };

        match (tipo.carregar)(componente.dados) {
            Ok(criado) => {
                if !cena.adicionar_box(id, criado) {
                    avisos.push(format!(
                        "{}: {} repetido, so o primeiro foi mantido",
                        salva.nome, tipo.nome
                    ));
                }
            }
            Err(erro) => avisos.push(format!(
                "{}: {} com dados invalidos, ignorado: {erro}",
                salva.nome, tipo.nome
            )),
        }
    }

    for filho in salva.filhos {
        recriar(cena, filho, Some(id), registro, avisos);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FormaColisor;
    use crate::settings::Color;
    use rugine_core::math::{Quat, Vec3};

    /// Uma cena com hierarquia, transforms nao triviais, os dois componentes
    /// e configuracoes fora do padrao.
    fn cena_de_teste() -> Scene {
        let mut cena = Scene::new();
        cena.settings.ambient.fog_density = 0.5;

        let pai = cena.spawn("Pai", None);
        cena.get_mut(pai).unwrap().transform = Transform {
            translation: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::from_rotation_y(0.7),
            scale: Vec3::new(1.0, 2.0, 0.5),
        };
        cena.adicionar(
            pai,
            Colisor {
                forma: FormaColisor::Esfera { raio: 0.8 },
                centro: Vec3::Y,
            },
        );

        let filho = cena.spawn("Filho", Some(pai));
        cena.get_mut(filho).unwrap().visible = false;
        cena.adicionar(
            filho,
            Malha {
                cor: Color::new(0.1, 0.2, 0.3),
            },
        );
        cena.adicionar(filho, Colisor::default());

        cena.spawn("Neto", Some(filho));
        cena.spawn("Outra raiz", None);
        cena
    }

    /// Descricao comparavel de uma cena inteira: nome, visibilidade,
    /// transform, componentes e profundidade, na ordem da arvore.
    fn descrever(cena: &Scene) -> Vec<String> {
        fn no(cena: &Scene, id: EntityId, nivel: usize, saida: &mut Vec<String>) {
            let e = cena.get(id).unwrap();
            let componentes: Vec<String> = cena
                .componentes(id)
                .iter()
                .map(|c| format!("{c:?}"))
                .collect();
            saida.push(format!(
                "{nivel} {} {} {:?} {componentes:?}",
                e.name, e.visible, e.transform
            ));
            for &filho in e.children() {
                no(cena, filho, nivel + 1, saida);
            }
        }

        let mut saida = vec![format!("{:?}", cena.settings)];
        for &raiz in cena.roots() {
            no(cena, raiz, 0, &mut saida);
        }
        saida
    }

    #[test]
    fn ida_e_volta_preserva_a_cena() {
        let registro = Registro::padrao();
        let original = cena_de_teste();

        let (texto, avisos) = salvar(&original, &registro);
        assert!(avisos.is_empty(), "{avisos:?}");

        let carregada = carregar(&texto, &registro).unwrap();
        assert!(carregada.avisos.is_empty(), "{:?}", carregada.avisos);
        assert_eq!(descrever(&carregada.cena), descrever(&original));
    }

    /// Salvar de novo o que foi carregado da o mesmo texto: nada muda so por
    /// passar pelo arquivo.
    #[test]
    fn salvar_e_estavel() {
        let registro = Registro::padrao();
        let (primeiro, _) = salvar(&cena_de_teste(), &registro);
        let carregada = carregar(&primeiro, &registro).unwrap();
        let (segundo, _) = salvar(&carregada.cena, &registro);
        assert_eq!(primeiro, segundo);
    }

    /// Componente que ninguem registrou: vira aviso, e o resto da entidade e
    /// da cena carrega.
    #[test]
    fn componente_desconhecido_vira_aviso() {
        let texto = r#"{
            "versao": 1,
            "entidades": [{
                "nome": "Caixa",
                "componentes": [
                    { "tipo": "Teleporte", "dados": {} },
                    { "tipo": "Malha", "dados": { "cor": { "r": 1.0, "g": 0.0, "b": 0.0 } } }
                ]
            }]
        }"#;

        let carregada = carregar(texto, &Registro::padrao()).unwrap();
        let id = carregada.cena.roots()[0];

        assert_eq!(carregada.avisos.len(), 1, "{:?}", carregada.avisos);
        assert!(carregada.avisos[0].contains("Teleporte"));
        assert_eq!(carregada.cena.componente::<Malha>(id).unwrap().cor.r, 1.0);
    }

    /// Um arquivo escrito a mao, so com o essencial: o que falta vem do
    /// padrao, e a entidade nasce visivel.
    #[test]
    fn campos_ausentes_usam_o_padrao() {
        let texto = r#"{ "versao": 1, "entidades": [{ "nome": "So o nome" }] }"#;

        let carregada = carregar(texto, &Registro::padrao()).unwrap();
        let id = carregada.cena.roots()[0];
        let entidade = carregada.cena.get(id).unwrap();

        assert!(entidade.visible);
        assert_eq!(entidade.transform, Transform::default());
        assert_eq!(carregada.cena.settings, SceneSettings::default());
    }

    #[test]
    fn versao_nova_e_recusada_com_erro_claro() {
        let texto = r#"{ "versao": 99, "entidades": "formato que nem conhecemos" }"#;
        let erro = carregar(texto, &Registro::padrao()).err().unwrap();
        assert!(
            matches!(erro, ErroArquivo::VersaoNova { encontrada: 99 }),
            "{erro}"
        );
    }

    #[test]
    fn texto_invalido_e_erro() {
        assert!(carregar("isto nao e json", &Registro::padrao()).is_err());
    }

    #[test]
    fn registro_cria_pelo_nome() {
        let registro = Registro::padrao();
        assert_eq!(registro.nomes().collect::<Vec<_>>(), ["Malha", "Colisor"]);
        assert_eq!(registro.criar("Colisor").unwrap().nome(), "Colisor");
        assert!(registro.criar("Teleporte").is_none());
    }

    #[test]
    #[should_panic(expected = "dois componentes registrados")]
    fn registrar_duas_vezes_e_erro() {
        let mut registro = Registro::padrao();
        registro.registrar::<Malha>();
    }
}
