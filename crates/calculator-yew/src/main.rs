use calculator_core::{
    eval::{Environment, Eval},
    expr::Expr,
    parse::parse_from_str,
};
use gloo_timers::callback::Timeout;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

enum Msg {
    ClickEvent,
    Focus,
    Blur,
    InputChanged(InputEvent),
    KeyboardEvent(KeyboardEvent),
    SelectionChangeEvent(Event),
}

struct App {
    input: String,
    outputs: Vec<String>,
    env: Environment,
    caret_pos: (usize, usize),
    input_element: Option<HtmlInputElement>,
    is_focused: bool,
}

impl App {
    fn input_element(&mut self) -> Option<&HtmlInputElement> {
        if self.input_element.is_none() {
            let input = web_sys::window()?
                .document()?
                .get_element_by_id("hidden-input")?
                .dyn_into::<HtmlInputElement>()
                .ok()?;
            self.input_element = Some(input);
        }
        self.input_element.as_ref()
    }
    fn update_caret_pos(&mut self, input: &HtmlInputElement) {
        let start = input.selection_start();
        let end = input.selection_end();
        self.caret_pos.0 = start.unwrap().unwrap_or_default() as _;
        self.caret_pos.1 = end
            .unwrap()
            .map(|end| self.input.len().min(end as _))
            .unwrap_or(self.input.len());
    }
    fn submit_input(&mut self, input: &HtmlInputElement) {
        self.outputs.push(format!("> {}", self.input));
        self.outputs
            .push(match parse_from_str::<Expr>(&self.input) {
                Ok(expr) => match expr.eval(&mut self.env) {
                    Ok(e) => format!("{}", e),
                    Err(err) => format!("error: {}", err),
                },
                Err(err) => {
                    format!("error: {}", err)
                }
            });
        self.input.clear();
        input.set_value("");
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            input: String::new(),
            outputs: Vec::new(),
            env: Environment::default(),
            caret_pos: (0, 0),
            input_element: None,
            is_focused: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ClickEvent => {
                let _ = self.input_element().unwrap().focus();
                false
            }
            Msg::Focus => {
                self.is_focused = true;
                true
            }
            Msg::Blur => {
                self.is_focused = false;
                true
            }
            Msg::InputChanged(event) => {
                let target = event.target().unwrap();
                let input = target.dyn_ref::<HtmlInputElement>().unwrap();
                self.input = input.value();
                self.update_caret_pos(input);
                true
            }
            Msg::KeyboardEvent(event) => match event.key().as_str() {
                "Enter" if !event.is_composing() => {
                    let target = event.target().unwrap();
                    self.submit_input(target.dyn_ref::<HtmlInputElement>().unwrap());
                    true
                }
                _ => {
                    let link = ctx.link().clone();
                    Timeout::new(1, move || {
                        link.send_message(Msg::SelectionChangeEvent(event.into()));
                    })
                    .forget();
                    false
                }
            },
            Msg::SelectionChangeEvent(event) => {
                let target = event.target().unwrap();
                self.update_caret_pos(target.dyn_ref::<HtmlInputElement>().unwrap());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let input_onkeydown = ctx
            .link()
            .callback(|e: KeyboardEvent| Msg::KeyboardEvent(e));
        let input_onclick = ctx.link().callback(|_e: MouseEvent| Msg::ClickEvent);
        let input_oninput = ctx
            .link()
            .callback(|event: InputEvent| Msg::InputChanged(event));
        let input_onselectionchange = ctx
            .link()
            .callback(|event: Event| Msg::SelectionChangeEvent(event));
        let input_onfocus = ctx.link().callback(|_e: FocusEvent| Msg::Focus);
        let input_onblur = ctx.link().callback(|_e: FocusEvent| Msg::Blur);
        let (left, right) = self.input.split_at(self.caret_pos.1.min(self.input.len()));
        let caret_classes = classes!("caret", self.is_focused.then(|| "is-focused"));

        html! {
            <main onclick={ input_onclick }>
                { for self.outputs.iter().map(|output| html!(<pre class="line">{ output }</pre>)) }
                <div class="input-area">
                    <pre class="input-cover">
                        { "> " }{ left }<span class={ caret_classes }></span>{ right }
                    </pre>
                    <input
                        type="text"
                        id="hidden-input"
                        oninput={ input_oninput }
                        onkeydown={ input_onkeydown }
                        onselectionchange={ input_onselectionchange }
                        onfocus={ input_onfocus }
                        onblur={ input_onblur }
                    />
                </div>
            </main>
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
