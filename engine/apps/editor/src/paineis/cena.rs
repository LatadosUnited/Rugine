//! Painel Cena: ceu, ambiente e pos-processamento.
//!
//! Hoje estes controles editam dados que o renderer ainda nao consome. Isso e
//! deliberado e esta dito na interface, porque um painel que parece funcionar
//! e nao faz nada e pior do que um painel que avisa.

use rugine_app::prelude::egui;
use rugine_scene::settings::SkyKind;

use crate::estado::Editor;
use crate::widgets;

pub fn desenhar(ui: &mut egui::Ui, editor: &mut Editor) {
    let settings = &mut editor.cena.settings;

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            widgets::secao(ui, "Ceu");

            widgets::propriedade(ui, "Tipo de ceu", |ui| {
                egui::ComboBox::from_id_salt("tipo_ceu")
                    .selected_text(settings.sky.kind.label())
                    .width(ui.available_width().min(120.0))
                    .show_ui(ui, |ui| {
                        for tipo in SkyKind::ALL {
                            ui.selectable_value(&mut settings.sky.kind, tipo, tipo.label());
                        }
                    });
            });

            widgets::propriedade(ui, "Ceu", |ui| {
                widgets::cor_rgb(ui, &mut settings.sky.color);
            });

            widgets::propriedade(ui, "Brilho do ceu", |ui| {
                ui.add(
                    egui::DragValue::new(&mut settings.sky.brightness)
                        .speed(0.01)
                        .range(0.0..=10.0)
                        .fixed_decimals(2),
                );
            });

            widgets::propriedade(ui, "Luz do ceu", |ui| {
                ui.add(
                    egui::DragValue::new(&mut settings.sky.light)
                        .speed(0.01)
                        .range(0.0..=10.0)
                        .fixed_decimals(2),
                );
            });

            ui.add_space(6.0);
            widgets::secao(ui, "Ambiente");

            widgets::propriedade(ui, "Neblina", |ui| {
                // Passo pequeno: a densidade e exponencial, e 0.01 ja e uma
                // mudanca grande no resultado.
                ui.add(
                    egui::DragValue::new(&mut settings.ambient.fog_density)
                        .speed(0.001)
                        .range(0.0..=1.0)
                        .fixed_decimals(4),
                );
            });

            widgets::propriedade(ui, "Luz ambiente", |ui| {
                widgets::cor_rgb(ui, &mut settings.ambient.light);
            });

            widgets::propriedade(ui, "Distancia", |ui| {
                ui.add(
                    egui::DragValue::new(&mut settings.ambient.fog_distance)
                        .speed(0.5)
                        .range(1.0..=10_000.0)
                        .suffix(" m")
                        .fixed_decimals(0),
                );
            });

            ui.add_space(6.0);
            widgets::secao(ui, "Pos-processamento");

            widgets::propriedade(ui, "Bloom", |ui| {
                ui.checkbox(&mut settings.post.bloom, "");
            });

            ui.add_enabled_ui(settings.post.bloom, |ui| {
                widgets::propriedade(ui, "Intensidade", |ui| {
                    ui.add(
                        egui::Slider::new(&mut settings.post.bloom_intensity, 0.0..=2.0)
                            .show_value(true),
                    );
                });
            });

            widgets::propriedade(ui, "Exposicao", |ui| {
                ui.add(egui::Slider::new(&mut settings.post.exposure, 0.1..=4.0).show_value(true));
            });

            widgets::propriedade(ui, "Vinheta", |ui| {
                ui.add(egui::Slider::new(&mut settings.post.vignette, 0.0..=1.0).show_value(true));
            });

            ui.add_space(10.0);
            ui.label(
                egui::RichText::new(
                    "Estes valores ainda nao chegam ao renderer: a Rugine nao tem \
                     iluminacao nem neblina. Eles editam a cena de verdade e passam \
                     a ter efeito quando o pipeline existir.",
                )
                .size(10.0)
                .color(widgets::ATENCAO),
            );
        });
}
