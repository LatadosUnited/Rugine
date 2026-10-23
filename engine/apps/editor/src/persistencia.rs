//! Abrir e salvar cenas pelo editor: dialogos, disco e o que mostrar ao
//! usuario. O formato em si fica em `rugine_scene::arquivo`.

use std::path::{Path, PathBuf};

use rugine_app::prelude::Context;
use rugine_scene::Scene;
use rugine_scene::arquivo::{self, EXTENSAO};

use crate::estado::Editor;

/// Troca a cena por uma vazia, sem arquivo.
pub fn nova(editor: &mut Editor) {
    trocar_cena(editor, Scene::new(), None);
}

/// Pergunta qual arquivo abrir e abre.
pub fn abrir(editor: &mut Editor, ctx: &Context<'_>) {
    let Some(caminho) = dialogo(editor, ctx).pick_file() else {
        return;
    };
    abrir_caminho(editor, &caminho);
}

/// Abre um arquivo conhecido, como o passado na linha de comando.
///
/// Falha deixa a cena atual intacta: o erro vira mensagem, e quem estava
/// editando nao perde o que tinha aberto.
pub fn abrir_caminho(editor: &mut Editor, caminho: &Path) {
    let texto = match std::fs::read_to_string(caminho) {
        Ok(texto) => texto,
        Err(erro) => {
            mostrar(
                editor,
                format!("Nao foi possivel ler {}:\n{erro}", caminho.display()),
            );
            return;
        }
    };

    match arquivo::carregar(&texto, &editor.registro) {
        Ok(carregada) => {
            trocar_cena(editor, carregada.cena, Some(caminho.to_path_buf()));
            log::info!("cena aberta: {}", caminho.display());
            if !carregada.avisos.is_empty() {
                mostrar(
                    editor,
                    format!(
                        "A cena abriu, mas parte dela ficou de fora:\n\n{}",
                        carregada.avisos.join("\n")
                    ),
                );
            }
        }
        Err(erro) => mostrar(
            editor,
            format!("Nao foi possivel abrir {}:\n{erro}", caminho.display()),
        ),
    }
}

/// Salva no arquivo atual, ou pergunta onde se a cena ainda nao tem um.
pub fn salvar(editor: &mut Editor, ctx: &Context<'_>) {
    match editor.arquivo.clone() {
        Some(caminho) => gravar(editor, &caminho),
        None => salvar_como(editor, ctx),
    }
}

/// Pergunta onde salvar e salva.
pub fn salvar_como(editor: &mut Editor, ctx: &Context<'_>) {
    let nome = editor
        .arquivo
        .as_deref()
        .and_then(Path::file_name)
        .map_or_else(
            || format!("cena.{EXTENSAO}"),
            |n| n.to_string_lossy().into_owned(),
        );

    let Some(caminho) = dialogo(editor, ctx).set_file_name(nome).save_file() else {
        return;
    };

    // Alguns dialogos devolvem o nome sem a extensao do filtro. Sem ela, o
    // arquivo nao apareceria no filtro na hora de abrir.
    let caminho = if caminho.extension().is_some_and(|e| e == EXTENSAO) {
        caminho
    } else {
        caminho.with_extension(EXTENSAO)
    };

    gravar(editor, &caminho);
}

/// Nome do arquivo atual, para a barra de menu.
pub fn titulo(editor: &Editor) -> String {
    editor
        .arquivo
        .as_deref()
        .and_then(Path::file_name)
        .map_or_else(
            || "Sem titulo".to_owned(),
            |n| n.to_string_lossy().into_owned(),
        )
}

fn gravar(editor: &mut Editor, caminho: &Path) {
    let (texto, avisos) = arquivo::salvar(&editor.cena, &editor.registro);

    if let Err(erro) = escrever_inteiro(caminho, &texto) {
        mostrar(
            editor,
            format!("Nao foi possivel salvar em {}:\n{erro}", caminho.display()),
        );
        return;
    }

    editor.arquivo = Some(caminho.to_path_buf());
    log::info!("cena salva: {}", caminho.display());

    if !avisos.is_empty() {
        mostrar(
            editor,
            format!("A cena foi salva, mas sem:\n\n{}", avisos.join("\n")),
        );
    }
}

/// Escreve num arquivo ao lado e so entao troca pelo de verdade.
///
/// Escrever direto no arquivo final deixaria uma cena pela metade se o
/// processo morresse no meio — e a cena boa anterior ja teria sido apagada.
/// Com a troca no fim, ou fica o arquivo antigo inteiro, ou o novo inteiro.
fn escrever_inteiro(caminho: &Path, texto: &str) -> std::io::Result<()> {
    let mut temporario = caminho.as_os_str().to_owned();
    temporario.push(".tmp");
    let temporario = PathBuf::from(temporario);

    std::fs::write(&temporario, texto)?;
    std::fs::rename(&temporario, caminho)
}

fn dialogo(editor: &Editor, ctx: &Context<'_>) -> rfd::FileDialog {
    let mut dialogo = rfd::FileDialog::new()
        .add_filter("Cena da Rugine", &[EXTENSAO])
        .set_parent(ctx.janela_dona());

    // Abre na pasta do arquivo atual: quem salva e abre costuma ficar na
    // mesma pasta do projeto.
    if let Some(pasta) = editor.arquivo.as_deref().and_then(Path::parent) {
        dialogo = dialogo.set_directory(pasta);
    }
    dialogo
}

/// Troca a cena inteira e limpa o que apontava para a antiga.
fn trocar_cena(editor: &mut Editor, cena: Scene, arquivo: Option<PathBuf>) {
    editor.cena = cena;
    editor.arquivo = arquivo;
    // Um arrasto ou destaque da cena anterior apontaria para entidades que
    // nao existem mais.
    editor.arrasto = None;
    editor.alca_sob_mouse = None;
}

fn mostrar(editor: &mut Editor, texto: String) {
    log::warn!("{texto}");
    editor.mensagem = Some(texto);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pasta_de_teste(nome: &str) -> PathBuf {
        let pasta =
            std::env::temp_dir().join(format!("rugine-teste-{nome}-{}", std::process::id()));
        std::fs::create_dir_all(&pasta).unwrap();
        pasta
    }

    /// O caminho inteiro pelo disco: a cena de exemplo e gravada, aberta de
    /// volta, e gravada de novo igual.
    #[test]
    fn cena_de_exemplo_vai_e_volta_pelo_disco() {
        let pasta = pasta_de_teste("ida-e-volta");
        let caminho = pasta.join(format!("exemplo.{EXTENSAO}"));

        let mut editor = Editor::default();
        let entidades = editor.cena.len();
        gravar(&mut editor, &caminho);
        assert!(editor.mensagem.is_none(), "{:?}", editor.mensagem);
        assert!(!caminho.with_extension(format!("{EXTENSAO}.tmp")).exists());

        let mut outro = Editor::default();
        nova(&mut outro);
        abrir_caminho(&mut outro, &caminho);
        assert!(outro.mensagem.is_none(), "{:?}", outro.mensagem);
        assert_eq!(outro.cena.len(), entidades);
        assert_eq!(outro.arquivo.as_deref(), Some(caminho.as_path()));

        let (primeiro, _) = arquivo::salvar(&editor.cena, &editor.registro);
        let (segundo, _) = arquivo::salvar(&outro.cena, &outro.registro);
        assert_eq!(primeiro, segundo);

        std::fs::remove_dir_all(pasta).ok();
    }

    /// Abrir algo que nao e cena avisa e nao toca na cena aberta.
    #[test]
    fn falha_ao_abrir_preserva_a_cena_atual() {
        let pasta = pasta_de_teste("invalido");
        let caminho = pasta.join(format!("quebrado.{EXTENSAO}"));
        std::fs::write(&caminho, "nao e json").unwrap();

        let mut editor = Editor::default();
        let antes = editor.cena.len();
        abrir_caminho(&mut editor, &caminho);

        assert!(editor.mensagem.is_some());
        assert_eq!(editor.cena.len(), antes);
        assert_eq!(editor.arquivo, None);

        std::fs::remove_dir_all(pasta).ok();
    }
}
