//! Painel Hierarquia: a arvore de entidades da cena.

use rugine_app::prelude::egui;
use rugine_scene::{EntityId, Scene};

use crate::estado::Editor;

/// O que a arvore pediu neste frame.
///
/// A arvore so *le* a cena enquanto desenha, e devolve a intencao. Mutar a
/// cena no meio da travessia exigiria emprestimo mutavel enquanto iteramos os
/// filhos — e, pior, mudaria a arvore sob os pes do proprio desenho.
#[derive(Default)]
struct Acoes {
    selecionar: Option<EntityId>,
    /// `Some(None)` cria na raiz; `Some(Some(id))` cria como filho de `id`.
    /// Um id sentinela de "sem pai" seria mais curto e mais facil de usar
    /// errado.
    criar_filho: Option<Option<EntityId>>,
    remover: Option<EntityId>,
}

pub fn desenhar(ui: &mut egui::Ui, editor: &mut Editor) {
    let mut acoes = Acoes::default();

    ui.horizontal(|ui| {
        if ui.button("+ Criar").clicked() {
            // Cria sob a selecao, que e o comportamento esperado quando ha
            // algo selecionado; na raiz quando nao ha.
            acoes.criar_filho = Some(editor.cena.selected());
        }

        ui.add(
            egui::TextEdit::singleline(&mut editor.busca_hierarquia)
                .hint_text("Buscar (Ctrl+F)")
                .desired_width(ui.available_width()),
        );
    });

    ui.add_space(4.0);

    let filtro = editor.busca_hierarquia.trim().to_lowercase();

    egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 1.0;

            for raiz in editor.cena.roots().to_vec() {
                no(ui, &editor.cena, raiz, &filtro, &mut acoes);
            }
        });

    aplicar(editor, acoes);
}

/// Desenha uma entidade e sua subarvore.
fn no(ui: &mut egui::Ui, cena: &Scene, id: EntityId, filtro: &str, acoes: &mut Acoes) {
    let Some(entidade) = cena.get(id) else {
        return;
    };

    // Um no fica visivel se ele casa com a busca ou se algum descendente casa:
    // esconder o pai esconderia o filho encontrado junto.
    if !filtro.is_empty() && !subarvore_casa(cena, id, filtro) {
        return;
    }

    let selecionado = cena.selected() == Some(id);
    let filhos = entidade.children();

    let rotulo = egui::RichText::new(&entidade.name)
        .size(13.0)
        .color(if entidade.visible {
            egui::Color32::from_gray(215)
        } else {
            // Invisivel na cena: apagado na arvore tambem.
            egui::Color32::from_gray(120)
        });

    if filhos.is_empty() {
        let resposta = ui.selectable_label(selecionado, rotulo);
        if resposta.clicked() {
            acoes.selecionar = Some(id);
        }
        menu_contexto(&resposta, id, acoes);
        return;
    }

    // Com filhos: cabecalho recolhivel. O id vem da entidade, nao do nome,
    // para que renomear nao recolha o no.
    let id_egui = ui.make_persistent_id(("no_hierarquia", format!("{id:?}")));
    egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id_egui, true)
        .show_header(ui, |ui| {
            let resposta = ui.selectable_label(selecionado, rotulo);
            if resposta.clicked() {
                acoes.selecionar = Some(id);
            }
            menu_contexto(&resposta, id, acoes);
        })
        .body(|ui| {
            for filho in filhos {
                no(ui, cena, *filho, filtro, acoes);
            }
        });
}

/// Menu de botao direito de uma entidade.
fn menu_contexto(resposta: &egui::Response, id: EntityId, acoes: &mut Acoes) {
    resposta.context_menu(|ui| {
        if ui.button("Criar filho").clicked() {
            acoes.criar_filho = Some(Some(id));
            ui.close();
        }
        if ui.button("Remover").clicked() {
            acoes.remover = Some(id);
            ui.close();
        }
    });
}

/// `true` se a entidade ou algum descendente casa com o filtro.
fn subarvore_casa(cena: &Scene, id: EntityId, filtro: &str) -> bool {
    let Some(entidade) = cena.get(id) else {
        return false;
    };

    if entidade.name.to_lowercase().contains(filtro) {
        return true;
    }

    entidade
        .children()
        .iter()
        .any(|filho| subarvore_casa(cena, *filho, filtro))
}

/// Aplica na cena o que a arvore pediu, agora que ninguem mais a le.
fn aplicar(editor: &mut Editor, acoes: Acoes) {
    if let Some(id) = acoes.selecionar {
        editor.cena.select(Some(id));
    }

    if let Some(pai) = acoes.criar_filho {
        // A selecao pode ter morrido entre o clique e aqui.
        let pai = pai.filter(|id| editor.cena.contains(*id));
        let novo = editor.cena.spawn("Entidade", pai);
        editor.cena.select(Some(novo));
    }

    if let Some(id) = acoes.remover {
        editor.cena.despawn(id);
    }
}
