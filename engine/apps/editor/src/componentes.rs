//! Como cada componente aparece no Inspector.
//!
//! Quais podem ser adicionados vem do `Registro` do `rugine-scene`, o mesmo que
//! salva e carrega: um catalogo so, e nao dois que poderiam divergir.
//!
//! Fica no editor, e nao no `rugine-scene`, porque e UI: a cena nao depende
//! de egui, e um jogo exportado nao precisa carregar os paineis.

use std::any::Any;

use rugine_app::prelude::{Vec3, egui};
use rugine_scene::{Colisor, Componente, FormaColisor, Malha};

use crate::widgets;

/// Os campos de um componente no Inspector.
pub fn desenhar(ui: &mut egui::Ui, componente: &mut dyn Componente) {
    let componente: &mut dyn Any = componente;

    if let Some(malha) = componente.downcast_mut::<Malha>() {
        desenhar_malha(ui, malha);
    } else if let Some(colisor) = componente.downcast_mut::<Colisor>() {
        desenhar_colisor(ui, colisor);
    } else {
        // Um componente sem editor ainda aparece, com nome e botao de
        // remover; so nao tem campos.
        ui.label(
            egui::RichText::new("Sem campos editaveis.")
                .size(11.0)
                .color(egui::Color32::from_gray(130)),
        );
    }
}

fn desenhar_malha(ui: &mut egui::Ui, malha: &mut Malha) {
    widgets::propriedade(ui, "Cor", |ui| {
        widgets::cor_rgb(ui, &mut malha.cor);
    });
}

fn desenhar_colisor(ui: &mut egui::Ui, colisor: &mut Colisor) {
    widgets::propriedade(ui, "Forma", |ui| {
        let atual = match colisor.forma {
            FormaColisor::Caixa { .. } => "Caixa",
            FormaColisor::Esfera { .. } => "Esfera",
        };

        egui::ComboBox::from_id_salt("forma_colisor")
            .selected_text(atual)
            .show_ui(ui, |ui| {
                // Trocar de forma converte o tamanho em vez de voltar ao
                // padrao: a esfera nova envolve a mesma regiao que a caixa
                // ocupava, e vice-versa.
                if ui.selectable_label(atual == "Caixa", "Caixa").clicked()
                    && let FormaColisor::Esfera { raio } = colisor.forma
                {
                    colisor.forma = FormaColisor::Caixa {
                        tamanho: Vec3::splat(raio * 2.0),
                    };
                }
                if ui.selectable_label(atual == "Esfera", "Esfera").clicked()
                    && let FormaColisor::Caixa { tamanho } = colisor.forma
                {
                    colisor.forma = FormaColisor::Esfera {
                        raio: tamanho.max_element() * 0.5,
                    };
                }
            });
    });

    widgets::propriedade(ui, "Centro", |ui| {
        let mut valores = colisor.centro.to_array();
        if widgets::campos_xyz(ui, &mut valores, 0.05) {
            colisor.centro = Vec3::from_array(valores);
        }
    });

    match &mut colisor.forma {
        FormaColisor::Caixa { tamanho } => {
            widgets::propriedade(ui, "Tamanho", |ui| {
                let mut valores = tamanho.to_array();
                if widgets::campos_xyz(ui, &mut valores, 0.05) {
                    // Tamanho negativo nao descreve volume nenhum.
                    *tamanho = Vec3::from_array(valores).max(Vec3::ZERO);
                }
            });
        }
        FormaColisor::Esfera { raio } => {
            widgets::propriedade(ui, "Raio", |ui| {
                ui.add(egui::DragValue::new(raio).speed(0.02).range(0.0..=f32::MAX));
            });
        }
    }
}
