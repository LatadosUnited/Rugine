import type { AreaId } from "./areas";

export interface Tecnologia {
  id: string;
  area: AreaId;
  /** Profundidade na arvore: quanto mais alto, mais tarde. Define a linha. */
  nivel: number;
  /** Pista dentro do nivel. So desempata quem fica ao lado de quem. */
  trilha: number;
  nome: string;
  /** Commit que trouxe. Sem ele, o estado sai dos pre-requisitos. */
  pronto?: string;
  req: string[];
  desc: string;
  efeitos: string[];
  /** Arquivos onde isto mora, relativos a raiz do repositorio. */
  onde?: string[];
}

export const TECNOLOGIAS: Tecnologia[] = [
  // ---------------- Nucleo ----------------
  {
    id: "workspace", area: "nucleo", nivel: 0, trilha: 0, nome: "Workspace e CI",
    pronto: "a8c6a02", req: [],
    desc: "Workspace Cargo com crates separadas (core, render, scene, ui, app) e dependências centralizadas. O CI compila e testa em Linux e Windows.",
    efeitos: [
      "Edition 2024, resolver 3",
      "CI só em pull request: no push ele demorava demais",
      "Linux verificado localmente com clippy --target x86_64-unknown-linux-gnu",
    ],
    onde: ["engine/Cargo.toml", ".github/workflows/ci.yml"],
  },
  {
    id: "grafo", area: "nucleo", nivel: 1, trilha: 1, nome: "Grafo de cena",
    pronto: "a8c6a02", req: ["workspace"],
    desc: "Entidades em arena, com hierarquia pai/filho e matriz de mundo composta pela cadeia de pais.",
    efeitos: [
      "EntityId com geração: id antigo não resolve para a entidade nova do mesmo slot",
      "Remover leva a descendência junto",
      "Seleção limpa quando o alvo morre",
    ],
    onde: ["engine/crates/rugine-scene/src/lib.rs"],
  },
  {
    id: "componentes", area: "nucleo", nivel: 7, trilha: 1, nome: "Componentes",
    pronto: "75ef89d", req: ["grafo"],
    desc: "Cada entidade guarda uma lista de componentes, na ordem do Inspector. Armazenamento próprio em vez de bevy_ecs: o editor precisa de acesso dinâmico pelo nome.",
    efeitos: [
      "Lista privada: toda mudança passa pela Scene, o funil do futuro desfazer",
      "Um componente de cada tipo por entidade",
      "com::<T>() percorre quem tem o componente",
    ],
    onde: ["engine/crates/rugine-scene/src/componentes.rs"],
  },
  {
    id: "colisor", area: "nucleo", nivel: 8, trilha: 2, nome: "Colisor",
    pronto: "75ef89d", req: ["componentes"],
    desc: "Caixa ou esfera no espaço local da entidade, com centro deslocável. Só é editado e desenhado: ainda nada colide.",
    efeitos: [
      "Arestas verdes conforme Exibir colliders",
      "Esfera sob escala não uniforme usa o maior eixo",
      "Trocar a forma converte o tamanho",
    ],
    onde: ["engine/crates/rugine-scene/src/componentes.rs", "engine/apps/editor/src/arestas.rs"],
  },
  {
    id: "registro", area: "nucleo", nivel: 8, trilha: 1, nome: "Registro de tipos",
    pronto: "7572dde", req: ["componentes"],
    desc: "Nome → criar, salvar e carregar cada tipo de componente. Um catálogo só para o arquivo, o menu de adicionar e, depois, o desfazer.",
    efeitos: ["Menu Adicionar componente lê do registro", "Nome repetido é recusado no registro"],
    onde: ["engine/crates/rugine-scene/src/componentes.rs"],
  },
  {
    id: "arquivo", area: "nucleo", nivel: 9, trilha: 1, nome: "Arquivo .cena",
    pronto: "7572dde", req: ["registro"],
    desc: "Cena salva em JSON com versão: hierarquia, transforms, componentes e configurações. JSON e não RON, porque o valor genérico do RON não representa enums.",
    efeitos: [
      "Campo ausente usa o padrão",
      "Componente desconhecido vira aviso, não falha",
      "Grava num .tmp e troca no fim",
      "editor arquivo.cena abre direto",
    ],
    onde: ["engine/crates/rugine-scene/src/arquivo.rs", "engine/apps/editor/src/persistencia.rs"],
  },
  {
    id: "luz", area: "nucleo", nivel: 10, trilha: 0, nome: "Componente Luz",
    req: ["registro"],
    desc: "Luz direcional ou pontual, com cor e intensidade, como componente de entidade.",
    efeitos: [
      "Libera a iluminação de verdade no renderer",
      "Entra no arquivo pelo registro, sem código novo de salvar",
    ],
  },
  {
    id: "camera-jogo", area: "nucleo", nivel: 10, trilha: 2, nome: "Componente Câmera",
    req: ["registro"],
    desc: "O ponto de vista do jogo, separado da câmera do editor.",
    efeitos: ["Pré-requisito do modo Play", "No editor, aparece como ícone na cena"],
  },
  {
    id: "fisica", area: "nucleo", nivel: 13, trilha: 2, nome: "Física",
    req: ["colisor", "play"],
    desc: "Os colisores passam a colidir. Precisa de um modo Play para ter um tempo de simulação separado da edição.",
    efeitos: [
      "Pode exigir trocar o armazenamento de componentes, se o runtime precisar de milhares de entidades por quadro",
    ],
  },
  {
    id: "scripts", area: "nucleo", nivel: 13, trilha: 0, nome: "Scripts",
    req: ["play"],
    desc: "Comportamento de jogo escrito pelo usuário. Hoje o painel Scripts e a compilação são simulados.",
    efeitos: ["Liga o painel Scripts e o F7 a algo real"],
  },

  // ---------------- Renderizacao ----------------
  {
    id: "janela", area: "render", nivel: 0, trilha: 1, nome: "Janela e triângulo",
    pronto: "a8c6a02", req: [],
    desc: "winit e wgpu 30 abrindo uma janela e desenhando. No Windows o padrão é DX12: o Vulkan da Intel HD 530 travava dentro do driver ao criar o device.",
    efeitos: ["WGPU_BACKEND sobrepõe o backend", "Convenção de NDC fixada num lugar só, com testes"],
    onde: ["engine/crates/rugine-app/src/janela.rs", "engine/crates/rugine-render/src/renderer.rs"],
  },
  {
    id: "viewport", area: "render", nivel: 1, trilha: 0, nome: "Viewport em textura",
    pronto: "a8c6a02", req: ["janela"],
    desc: "A cena é desenhada numa textura própria, e o viewport vira um painel como os outros.",
    efeitos: ["Duas vistas da janela: sRGB para a cena, linear para a UI, sem gama em dobro"],
    onde: ["engine/crates/rugine-render/src/target.rs", "engine/apps/editor/src/paineis/viewport.rs"],
  },
  {
    id: "profundidade", area: "render", nivel: 2, trilha: 0, nome: "Câmera e profundidade",
    pronto: "4b8401d", req: ["viewport"],
    desc: "Câmera por uniform e depth buffer Depth32Float. Mundo right-handed, Y para cima.",
    efeitos: ["Objetos se escondem atrás uns dos outros", "Grade com teste de profundidade sem escrita"],
    onde: ["engine/crates/rugine-render/src/depth.rs", "engine/crates/rugine-core/src/math.rs"],
  },
  {
    id: "instancias", area: "render", nivel: 3, trilha: 1, nome: "Instâncias",
    pronto: "4b8401d", req: ["profundidade"],
    desc: "Uma instância por entidade numa única chamada de desenho. A matriz de modelo vai como quatro vec4.",
    efeitos: ["Editar a transform no Inspector move o objeto no viewport"],
    onde: [
      "engine/crates/rugine-render/src/cena_gpu.rs",
      "engine/crates/rugine-render/src/shaders/malha.wgsl",
    ],
  },
  {
    id: "vsync", area: "render", nivel: 3, trilha: 2, nome: "Vsync alternável",
    pronto: "9c1d3b0", req: ["janela"],
    desc: "Vsync ligado por padrão e desligável em execução, para medir o custo real do quadro.",
    efeitos: [
      "Medido: 60 fps com vsync, 300–530 sem, na HD 530",
      "Controle cinza quando o driver não oferece modo sem sincronização",
    ],
    onde: ["engine/crates/rugine-render/src/target.rs"],
  },
  {
    id: "sobreposicao", area: "render", nivel: 4, trilha: 1, nome: "Camada sobreposta",
    pronto: "a9c97cb", req: ["instancias"],
    desc: "Pipeline sem teste de profundidade para o que fica por cima de tudo, como os gizmos.",
    efeitos: [
      "Buffer próprio: no DX12 desta máquina, instâncias lidas de um deslocamento no meio do buffer não renderizavam nada, sem erro",
    ],
    onde: ["engine/crates/rugine-render/src/pipelines.rs"],
  },
  {
    id: "captura", area: "render", nivel: 6, trilha: 2, nome: "Captura do viewport",
    pronto: "a9c97cb", req: ["viewport"],
    desc: "Lê o viewport de volta da GPU e salva em PNG. Foi o que achou a causa dos gizmos invisíveis.",
    efeitos: [
      "RUGINE_CAPTURA=saida.png",
      "Conta pixels coloridos no log",
      "Também revelou o viewport em 163 px esticado",
    ],
    onde: ["engine/crates/rugine-render/src/captura.rs"],
  },
  {
    id: "normais", area: "render", nivel: 10, trilha: 1, nome: "Normais corretas",
    req: ["instancias"],
    desc: "O shader usa a matriz de modelo direto, o que entorta a normal sob escala não uniforme. O certo é a transposta da inversa.",
    efeitos: ["Iluminação certa em objetos esticados", "Custa um dado a mais por instância"],
  },
  {
    id: "teste-pixel", area: "render", nivel: 10, trilha: 4, nome: "Teste de pixel",
    req: ["captura"],
    desc: "A captura vira asserção: cada mudança visual confere a imagem automaticamente.",
    efeitos: ["Exige GPU, então não roda nos runners do CI"],
  },
  {
    id: "iluminacao", area: "render", nivel: 11, trilha: 0, nome: "Iluminação e neblina",
    req: ["profundidade", "luz"],
    desc: "Luzes de verdade e neblina. Liga os controles do painel Cena, que hoje editam dados que ninguém lê.",
    efeitos: ["A luz fixa no shader sai", "Céu, ambiente e neblina passam a valer"],
  },
  {
    id: "gltf", area: "render", nivel: 11, trilha: 1, nome: "Malhas de glTF",
    req: ["instancias", "arquivo"],
    desc: "A Malha deixa de ser sempre o cubo e passa a referenciar um arquivo glTF.",
    efeitos: [
      "Uma lista de instâncias por malha",
      "Picking por caixa envolvente",
      "Dá conteúdo à aba Projeto",
    ],
  },
  {
    id: "materiais", area: "render", nivel: 12, trilha: 1, nome: "Materiais e texturas",
    req: ["gltf"],
    desc: "Texturas e parâmetros de superfície vindos do glTF, no lugar da cor única da Malha.",
    efeitos: [],
  },
  {
    id: "sombras", area: "render", nivel: 13, trilha: 1, nome: "Sombras",
    req: ["iluminacao"],
    desc: "Mapas de sombra para as luzes direcionais e pontuais.",
    efeitos: [],
  },
  {
    id: "pos", area: "render", nivel: 13, trilha: 3, nome: "Pós-processamento",
    req: ["iluminacao"],
    desc: "Bloom, exposição e vinheta, que o painel Cena já edita mas o renderer ainda ignora.",
    efeitos: [],
  },

  // ---------------- Editor ----------------
  {
    id: "ui", area: "editor", nivel: 0, trilha: 2, nome: "Interface do editor",
    pronto: "a8c6a02", req: [],
    desc: "Menu, Hierarquia, Inspector, Cena, Projeto e Scripts em egui, no tema escuro e com ícones desenhados.",
    efeitos: [
      "Tema escuro forçado: seguir o tema do sistema deixava a UI branca",
      "Ícones pintados no lugar de glifos que viravam quadrados",
    ],
    onde: [
      "engine/crates/rugine-ui/src/lib.rs",
      "engine/apps/editor/src/paineis/mod.rs",
      "engine/apps/editor/src/widgets.rs",
    ],
  },
  {
    id: "paineis", area: "editor", nivel: 1, trilha: 2, nome: "Painéis encaixáveis",
    pronto: "a8c6a02", req: ["ui"],
    desc: "Qualquer painel pode virar aba de outro, ir para outro canto ou flutuar.",
    efeitos: ["egui_dock, com a egui presa na 0.36"],
    onde: ["engine/apps/editor/src/docking.rs"],
  },
  {
    id: "barra", area: "editor", nivel: 1, trilha: 3, nome: "Barra de título própria",
    pronto: "a8c6a02", req: ["ui"],
    desc: "Barra de título no tema do editor, com arrastar e redimensionar pelas bordas.",
    efeitos: [
      "Windows e Linux",
      "macOS fica com a barra nativa: lá a winit não redimensiona janela sem decoração",
    ],
    onde: ["engine/apps/editor/src/paineis/bordas.rs", "engine/crates/rugine-app/src/janela.rs"],
  },
  {
    id: "navegacao", area: "editor", nivel: 3, trilha: 0, nome: "Navegação estilo Unity",
    pronto: "6faef61", req: ["profundidade"],
    desc: "Botão direito olha em volta, com WASD voa, Alt orbita, o meio desloca e F enquadra a seleção.",
    efeitos: ["Pivô e distância: olhar move o pivô, orbitar move a câmera"],
    onde: ["engine/apps/editor/src/camera.rs", "engine/apps/editor/src/paineis/interacao.rs"],
  },
  {
    id: "picking", area: "editor", nivel: 4, trilha: 0, nome: "Picking",
    pronto: "bd54357", req: ["profundidade", "grafo"],
    desc: "Clique seleciona pelo raio contra a caixa de cada entidade, na CPU, sem passe extra na GPU.",
    efeitos: ["Raio levado ao espaço local: origem com w=1, direção com w=0"],
    onde: ["engine/apps/editor/src/picking.rs", "engine/crates/rugine-core/src/raio.rs"],
  },
  {
    id: "gizmos", area: "editor", nivel: 5, trilha: 0, nome: "Gizmos de mover e escalar",
    pronto: "bd54357", req: ["picking", "sobreposicao"],
    desc: "Hastes por eixo, agarradas pelo mesmo teste de raio do picking.",
    efeitos: ["Tamanho constante na tela, perto ou longe", "Snap configurável"],
    onde: ["engine/apps/editor/src/gizmo.rs"],
  },
  {
    id: "rotacao", area: "editor", nivel: 6, trilha: 0, nome: "Gizmo de rotação",
    pronto: "1a56d5e", req: ["gizmos"],
    desc: "Anéis por eixo e os modos Global e Local.",
    efeitos: ["Ângulo acumulado: várias voltas sem salto em 180°", "Snap de 15°"],
    onde: ["engine/apps/editor/src/gizmo.rs"],
  },
  {
    id: "arrasto", area: "editor", nivel: 6, trilha: 1, nome: "Arrasto estável e escala uniforme",
    pronto: "2ec23f7", req: ["gizmos"],
    desc: "O gizmo fica congelado durante o arrasto, e o cubo do centro escala os três eixos juntos.",
    efeitos: [
      "Acabou o pisca: o deslocamento era medido contra uma origem que andava junto",
      "Escala multiplicada, mantendo a proporção",
    ],
    onde: ["engine/apps/editor/src/gizmo.rs"],
  },
  {
    id: "inspector", area: "editor", nivel: 8, trilha: 0, nome: "Inspector de componentes",
    pronto: "75ef89d", req: ["componentes"],
    desc: "Um bloco recolhível por componente, com remover, e o menu de adicionar.",
    efeitos: ["Linhas cabem no painel a partir de 230 px, com teste sem GPU"],
    onde: [
      "engine/apps/editor/src/paineis/inspector.rs",
      "engine/apps/editor/src/componentes.rs",
    ],
  },
  {
    id: "contorno", area: "editor", nivel: 8, trilha: 3, nome: "Contorno de seleção",
    pronto: "75ef89d", req: ["componentes"],
    desc: "A seleção vira contorno laranja nas arestas, e o objeto mantém a cor da Malha.",
    efeitos: ["Mudar a cor aparece com o objeto selecionado"],
    onde: ["engine/apps/editor/src/arestas.rs"],
  },
  {
    id: "desfazer", area: "editor", nivel: 10, trilha: 3, nome: "Desfazer e refazer",
    req: ["registro"],
    desc: "Histórico de comandos. O funil da Scene já existe; falta passar transform, nome e visibilidade por ele.",
    efeitos: ["Ctrl+Z e Ctrl+Y no menu Editar", "O registro recria um componente removido"],
  },
  {
    id: "projeto", area: "editor", nivel: 11, trilha: 2, nome: "Navegador de projeto",
    req: ["arquivo"],
    desc: "A aba Projeto lista as cenas e arquivos da pasta, e abre com duplo clique.",
    efeitos: [],
  },
  {
    id: "duplicar", area: "editor", nivel: 11, trilha: 3, nome: "Duplicar entidade",
    req: ["registro"],
    desc: "Ctrl+D copia a entidade com os filhos e os componentes.",
    efeitos: ["Componentes copiados pelo registro"],
  },
  {
    id: "play", area: "editor", nivel: 12, trilha: 0, nome: "Modo Play",
    req: ["camera-jogo", "arquivo"],
    desc: "Rodar o jogo dentro do editor e voltar à cena de antes ao parar.",
    efeitos: ["A cena é guardada como no arquivo e restaurada ao sair", "Libera física e scripts"],
  },
];

/** A ordem recomendada da proxima pesquisa. */
export const FILA = ["desfazer", "luz", "iluminacao", "gltf"];
