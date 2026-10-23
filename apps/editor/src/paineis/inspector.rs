//! Painel Inspector: propriedades da entidade selecionada.

use rugine_app::prelude::egui;
use rugine_core::math::{EulerRot, Quat, Vec3};

use rugine_scene::EntityId;

use crate::componentes;
use crate::estado::Editor;
use crate::widgets;

pub fn desenhar(ui: &mut egui::Ui, editor: &mut Editor) {
    let Some(id) = editor.cena.selected() else {
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("Nenhuma entidade selecionada.")
                .size(12.0)
                .color(egui::Color32::from_gray(140)),
        );
        return;
    };

    if !editor.cena.contains(id) {
        return;
    }

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| conteudo(ui, editor, id));
}

/// Tudo que o Inspector mostra, fora da area de rolagem.
///
/// Separado para os testes medirem a largura do conteudo: dentro da rolagem,
/// o que passa da borda fica cortado e nao aparece na medida.
fn conteudo(ui: &mut egui::Ui, editor: &mut Editor, id: EntityId) {
    // Fase 1: edicao. O emprestimo mutavel da entidade vive so neste
    // escopo, porque a fase 2 precisa ler a cena inteira para compor a
    // matriz de mundo — e os dois emprestimos nao coexistem.
    {
        let Some(entidade) = editor.cena.get_mut(id) else {
            return;
        };

        ui.horizontal(|ui| {
            ui.checkbox(&mut entidade.visible, "");
            ui.add(
                egui::TextEdit::singleline(&mut entidade.name).desired_width(ui.available_width()),
            );
        });

        ui.add_space(6.0);
        ui.separator();
        ui.add_space(4.0);

        ui.label(
            egui::RichText::new("Transform")
                .size(12.0)
                .color(egui::Color32::from_gray(190)),
        );
        ui.add_space(4.0);

        let transform = &mut entidade.transform;

        widgets::propriedade(ui, "Posicao", |ui| {
            let mut valores = transform.translation.to_array();
            if widgets::campos_xyz(ui, &mut valores, 0.05) {
                transform.translation = Vec3::from_array(valores);
            }
        });

        widgets::propriedade(ui, "Rotacao", |ui| {
            // Graus de Euler na UI, quaternion no modelo. Euler e
            // legivel para quem edita; quaternion e o que nao sofre
            // gimbal lock nem acumula erro ao compor.
            //
            // A ida e volta nao e exata: varios trios de Euler dao a
            // mesma rotacao, entao os numeros podem saltar ao cruzar
            // 180 graus. So reescrevemos o quaternion quando o usuario
            // mexe, para que o campo parado nao fique se reescrevendo.
            let (y, x, z) = transform.rotation.to_euler(EulerRot::YXZ);
            let mut graus = [x.to_degrees(), y.to_degrees(), z.to_degrees()];

            if widgets::campos_xyz(ui, &mut graus, 0.5) {
                transform.rotation = Quat::from_euler(
                    EulerRot::YXZ,
                    graus[1].to_radians(),
                    graus[0].to_radians(),
                    graus[2].to_radians(),
                );
            }
        });

        widgets::propriedade(ui, "Escala", |ui| {
            let mut valores = transform.scale.to_array();
            if widgets::campos_xyz(ui, &mut valores, 0.01) {
                transform.scale = Vec3::from_array(valores);
            }
        });
    }

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    // Fase 2: leitura. A matriz de mundo e util para conferir se a
    // hierarquia compoe como se espera — e a primeira coisa que se
    // quer ver quando um filho aparece no lugar errado.
    let mundo = editor.cena.world_matrix(id);
    let posicao = mundo.transform_point3(Vec3::ZERO);
    ui.label(
        egui::RichText::new(format!(
            "Mundo: {:.2}, {:.2}, {:.2}",
            posicao.x, posicao.y, posicao.z
        ))
        .size(11.0)
        .color(egui::Color32::from_gray(130)),
    );

    lista_de_componentes(ui, editor, id);

    ui.add_space(8.0);
    menu_adicionar(ui, editor, id);
}

/// Um bloco recolhivel por componente, na ordem em que foram adicionados.
fn lista_de_componentes(ui: &mut egui::Ui, editor: &mut Editor, id: EntityId) {
    // A remocao e adiada para depois do laco: remover no meio dele mexeria
    // na lista que esta sendo percorrida.
    let mut remover = None;

    for (indice, componente) in editor.cena.componentes_mut(id).iter_mut().enumerate() {
        let nome = componente.nome();

        ui.add_space(4.0);
        ui.separator();

        // O id pelo nome do tipo, e nao pelo indice: assim remover o
        // primeiro componente nao faz o segundo herdar o estado
        // aberto/fechado dele.
        let estado = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id(("componente", nome)),
            true,
        );

        estado
            .show_header(ui, |ui| {
                ui.label(
                    egui::RichText::new(nome)
                        .size(12.0)
                        .color(egui::Color32::from_gray(190)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button("x")
                        .on_hover_text("Remover componente")
                        .clicked()
                    {
                        remover = Some(indice);
                    }
                });
            })
            .body(|ui| componentes::desenhar(ui, componente.as_mut()));
    }

    if let Some(indice) = remover {
        editor.cena.remover_indice(id, indice);
    }
}

/// O botao que lista os componentes que esta entidade ainda nao tem.
fn menu_adicionar(ui: &mut egui::Ui, editor: &mut Editor, id: EntityId) {
    ui.menu_button("+ Adicionar componente", |ui| {
        for tipo in &componentes::ADICIONAVEIS {
            let nome = tipo.nome();

            // Desabilitado em vez de escondido: o item some do lugar de
            // sempre e confunde mais do que um item cinza.
            let ja_tem = editor.cena.componentes(id).iter().any(|c| c.nome() == nome);

            if ui.add_enabled(!ja_tem, egui::Button::new(nome)).clicked() {
                editor.cena.adicionar_box(id, (tipo.criar)());
                ui.close();
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rugine_scene::Malha;

    /// Largura que o conteudo ocupa quando so ha `largura` disponivel.
    fn largura_ocupada(editor: &mut Editor, id: EntityId, largura: f32) -> f32 {
        let ctx = egui::Context::default();
        let entrada = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(largura + 100.0, 2000.0),
            )),
            ..Default::default()
        };

        let mut ocupada = 0.0;
        let mut saida = ctx.run_ui(entrada, |ui| {
            ui.allocate_ui(egui::vec2(largura, 2000.0), |ui| {
                ui.set_max_width(largura);
                conteudo(ui, editor, id);
                ocupada = ui.min_rect().width();
            });
        });
        // Sem GPU para aplicar o atlas de fontes; a epaint entra em panico
        // ao descartar deltas nao aplicados.
        saida.textures_delta.clear();
        ocupada
    }

    /// Nada no Inspector pode ser mais largo que o painel: o que passa vira
    /// uma barra de rolagem lateral num painel que so deveria rolar para
    /// baixo.
    #[test]
    fn conteudo_cabe_na_largura_do_painel() {
        let mut editor = Editor::default();
        let com_malha = editor.cena.com::<Malha>().next().unwrap().0;
        let sem_malha = editor.cena.selected().unwrap();

        let mut estouros = Vec::new();
        for id in [com_malha, sem_malha] {
            let nome = editor.cena.get(id).unwrap().name.clone();
            // De 230 para cima. Abaixo de ~226 px nao ha conta que resolva:
            // um campo com "-90.00" nao encolhe alem do proprio numero, e tres
            // deles lado a lado deixam de caber. O painel abre com ~340.
            for largura in [230.0, 260.0, 300.0, 340.0, 400.0] {
                let ocupada = largura_ocupada(&mut editor, id, largura);
                if ocupada > largura + 0.5 {
                    estouros.push(format!("{nome} em {largura}: ocupou {ocupada:.1}"));
                }
            }
        }

        assert!(estouros.is_empty(), "{estouros:#?}");
    }
}
