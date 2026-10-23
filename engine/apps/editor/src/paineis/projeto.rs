//! Painel inferior: abas Projeto e Scripts.

use rugine_app::prelude::egui;

use crate::estado::{Editor, EstadoCompilacao};
use crate::widgets;

pub fn projeto(ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.label(
                egui::RichText::new(
                    "Sem pipeline de assets ainda. Esta aba lista arquivos do projeto \
                     quando houver importacao.",
                )
                .size(11.0)
                .color(egui::Color32::from_gray(140)),
            );
        });
}

pub fn scripts(ui: &mut egui::Ui, editor: &mut Editor) {
    ui.horizontal(|ui| {
        // Recompilar fica desabilitado durante a compilacao: enfileirar uma
        // segunda so produz confusao sobre qual resultado e de qual pedido.
        let compilando = matches!(editor.compilacao, EstadoCompilacao::Compilando { .. });

        ui.add_enabled_ui(!compilando, |ui| {
            if ui.button("Recompilar (F7)").clicked() {
                editor.compilacao = EstadoCompilacao::Compilando { segundos: 0.0 };
            }
        });

        if ui.button("Novo script...").clicked() {
            log::info!("criacao de script ainda nao implementada");
        }

        if ui.button("Abrir pasta").clicked() {
            log::info!("abertura de pasta ainda nao implementada");
        }
    });

    ui.add_space(6.0);

    match &editor.compilacao {
        EstadoCompilacao::Compilando { segundos } => {
            ui.label(
                egui::RichText::new(format!("Compilando... {segundos:.1} s"))
                    .size(12.0)
                    .color(widgets::ATENCAO),
            );
        }
        EstadoCompilacao::Ocioso => {}
    }

    ui.add_space(4.0);
    widgets::secao(ui, "Scripts");

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            for script in &editor.scripts {
                ui.label(
                    egui::RichText::new(script)
                        .size(12.0)
                        .color(egui::Color32::from_gray(200)),
                );
            }

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("Carregados: nenhuma DLL")
                    .size(11.0)
                    .color(egui::Color32::from_gray(130)),
            );
        });
}
