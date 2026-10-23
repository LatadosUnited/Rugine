//! Barra de menu superior.

use rugine_app::prelude::{Context, egui};

use crate::estado::{Editor, EstadoCompilacao};
use crate::persistencia;
use crate::widgets;

pub fn desenhar(ui: &mut egui::Ui, editor: &mut Editor, ctx: &mut Context<'_>) {
    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("Arquivo", |ui| {
            if ui.button("Nova cena").clicked() {
                persistencia::nova(editor);
                ui.close();
            }
            if ui
                .add(egui::Button::new("Abrir...").shortcut_text("Ctrl+O"))
                .clicked()
            {
                // Fechar o menu antes: o dialogo nativo segura o quadro ate
                // ser fechado, e o menu ficaria aberto por baixo dele.
                ui.close();
                persistencia::abrir(editor, ctx);
            }
            ui.separator();
            if ui
                .add(egui::Button::new("Salvar").shortcut_text("Ctrl+S"))
                .clicked()
            {
                ui.close();
                persistencia::salvar(editor, ctx);
            }
            if ui
                .add(egui::Button::new("Salvar como...").shortcut_text("Ctrl+Shift+S"))
                .clicked()
            {
                ui.close();
                persistencia::salvar_como(editor, ctx);
            }
            ui.separator();
            if ui.button("Sair").clicked() {
                ctx.exit();
                ui.close();
            }
        });

        ui.menu_button("Editar", |ui| {
            // Desfazer precisa de um historico de comandos, que e uma decisao
            // de arquitetura por si so: toda mutacao de cena teria que passar
            // por comandos. Fica para quando o ECS entrar.
            ui.add_enabled(false, egui::Button::new("Desfazer (Ctrl+Z)"));
            ui.add_enabled(false, egui::Button::new("Refazer (Ctrl+Y)"));
        });

        ui.menu_button("Entidade", |ui| {
            if ui.button("Criar vazia").clicked() {
                let novo = editor.cena.spawn("Entidade", None);
                editor.cena.select(Some(novo));
                ui.close();
            }

            let selecionada = editor.cena.selected();
            ui.add_enabled_ui(selecionada.is_some(), |ui| {
                if ui.button("Remover selecionada").clicked() {
                    if let Some(id) = selecionada {
                        editor.cena.despawn(id);
                    }
                    ui.close();
                }
            });
        });

        ui.menu_button("Ver", |ui| {
            ui.checkbox(&mut editor.grade, "Grade");
            ui.checkbox(&mut editor.exibir_ui, "UI");
            ui.checkbox(&mut editor.exibir_estatisticas, "Estatisticas");

            ui.separator();
            vsync(ui, ctx);
        });

        ui.menu_button("Jogo", |ui| {
            ui.add_enabled(false, egui::Button::new("Jogar (F5)"));
            ui.add_enabled(false, egui::Button::new("Pausar"));
        });

        ui.menu_button("Scripts", |ui| {
            if ui.button("Recompilar (F7)").clicked() {
                editor.compilacao = EstadoCompilacao::Compilando { segundos: 0.0 };
                ui.close();
            }
        });

        ui.menu_button("Ajuda", |ui| {
            ui.label(
                egui::RichText::new("Rugine - Marco 2")
                    .size(12.0)
                    .color(egui::Color32::from_gray(180)),
            );
        });

        // Status a direita dos menus, como no editor de referencia.
        if let EstadoCompilacao::Compilando { segundos } = editor.compilacao {
            ui.add_space(12.0);
            ui.label(
                egui::RichText::new(format!("Compilando scripts... {segundos:.0} s"))
                    .size(12.0)
                    .color(widgets::ATENCAO),
            );
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if editor.barra_propria {
                controles_janela(ui, ctx);
                ui.add_space(8.0);
            }

            ui.label(
                egui::RichText::new(format!(
                    "{}  |  {} entidades",
                    persistencia::titulo(editor),
                    editor.cena.len()
                ))
                .size(11.0)
                .color(egui::Color32::from_gray(130)),
            );

            // O espaco que sobra entre o menu e os botoes vira a area de
            // arrastar. E o que substitui a barra de titulo do sistema.
            if editor.barra_propria {
                area_de_arrastar(ui, ctx);
            }
        });
    });
}

/// Alternancia de vsync.
///
/// Desabilitado quando a surface nao oferece nenhum modo sem sincronizacao:
/// o `AutoNoVsync` do wgpu cairia para Fifo e o controle nao faria nada. Um
/// checkbox que muda de estado sem mudar o comportamento e pior do que um
/// checkbox cinza com o motivo no tooltip.
fn vsync(ui: &mut egui::Ui, ctx: &mut Context<'_>) {
    let disponivel = ctx.renderer.sem_vsync_disponivel();
    let mut ligado = ctx.renderer.vsync();

    let resposta = ui
        .add_enabled(disponivel, egui::Checkbox::new(&mut ligado, "Vsync"))
        .on_disabled_hover_text(
            "Esta GPU/driver so oferece apresentacao sincronizada nesta janela.",
        )
        .on_hover_text("Desligue para medir o custo real do frame.");

    if resposta.changed() {
        ctx.renderer.definir_vsync(ligado);
    }
}

/// Minimizar, maximizar/restaurar e fechar.
fn controles_janela(ui: &mut egui::Ui, ctx: &mut Context<'_>) {
    // Layout da direita para a esquerda, entao fechar vem primeiro para
    // terminar na quina — a posicao que todo mundo ja conhece.
    if widgets::botao_janela(ui, widgets::IconeJanela::Fechar)
        .on_hover_text("Fechar")
        .clicked()
    {
        ctx.exit();
    }

    let maximizada = ctx.janela_maximizada();
    let icone = if maximizada {
        widgets::IconeJanela::Restaurar
    } else {
        widgets::IconeJanela::Maximizar
    };

    if widgets::botao_janela(ui, icone)
        .on_hover_text(if maximizada { "Restaurar" } else { "Maximizar" })
        .clicked()
    {
        ctx.alternar_maximizada();
    }

    if widgets::botao_janela(ui, widgets::IconeJanela::Minimizar)
        .on_hover_text("Minimizar")
        .clicked()
    {
        ctx.minimizar_janela();
    }
}

/// A faixa vazia da barra que move a janela.
fn area_de_arrastar(ui: &mut egui::Ui, ctx: &mut Context<'_>) {
    let espaco = ui.available_size();
    if espaco.x <= 0.0 {
        return;
    }

    let (_rect, resposta) = ui.allocate_exact_size(espaco, egui::Sense::click_and_drag());

    // `drag_started`, e nao `dragged`: `drag_window` entrega o laco de
    // movimento ao sistema, que conduz o arrasto ate o botao ser solto.
    // Chamar a cada frame reiniciaria esse laco sem parar.
    if resposta.drag_started() {
        ctx.arrastar_janela();
    }

    // Duplo clique alterna maximizada, como na barra nativa.
    if resposta.double_clicked() {
        ctx.alternar_maximizada();
    }
}
