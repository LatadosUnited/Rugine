//! Painel Inspector: propriedades da entidade selecionada.

use rugine_app::prelude::egui;
use rugine_core::math::{EulerRot, Quat, Vec3};

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
        .show(ui, |ui| {
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
                        egui::TextEdit::singleline(&mut entidade.name)
                            .desired_width(ui.available_width()),
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

            ui.add_space(8.0);
            if ui.button("+ Adicionar componente").clicked() {
                // Sem sistema de componentes ainda. O botao existe porque a
                // forma do painel depende dele; ligar vem com o ECS.
                log::info!("componentes ainda nao existem");
            }
        });
}
