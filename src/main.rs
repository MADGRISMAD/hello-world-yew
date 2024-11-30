use yew::prelude::*;
use serde::{Deserialize, Serialize};
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct MongoId {
    #[serde(rename = "$oid")]
    oid: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct Student {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<MongoId>,
    name: String,
    age: u8,
    subject: String,
}

#[function_component(App)]
fn app() -> Html {
    let students = use_state(|| Rc::new(RefCell::new(vec![])));
    let name = use_state(String::new);
    let age = use_state(String::new);
    let subject = use_state(String::new);
    let editing_student = use_state(|| None::<Student>);

    {
        let students = students.clone();
        use_effect_with_deps(
            move |_| {
                let students = students.clone();
                spawn_local(async move {
                    fetch_students(students).await;
                });
                || ()
            },
            (),
        );
    }

    let add_student = {
        let name = name.clone();
        let age = age.clone();
        let subject = subject.clone();
        let students = students.clone();

        Callback::from(move |_: MouseEvent| {
            let name = name.clone();
            let age = age.clone();
            let subject = subject.clone();
            let students = students.clone();

            spawn_local(async move {
                let student = Student {
                    id: None,
                    name: (*name).clone(),
                    age: (*age).clone().parse::<u8>().unwrap_or(0),
                    subject: (*subject).clone(),
                };

                if let Err(err) = Request::post("http://localhost:8000/students")
                    .json(&student)
                    .unwrap()
                    .send()
                    .await
                {
                    gloo::console::log!(format!("Error al agregar estudiante: {:?}", err));
                }

                fetch_students(students.clone()).await;

                name.set(String::new());
                age.set(String::new());
                subject.set(String::new());
            });
        })
    };

    let delete_student = {
        let students = students.clone();

        Callback::from(move |id: String| {
            let students = students.clone();
            spawn_local(async move {
                if let Err(err) = Request::delete(&format!("http://localhost:8000/students/{}", id))
                    .send()
                    .await
                {
                    gloo::console::log!(format!("Error al eliminar estudiante: {:?}", err));
                }
                fetch_students(students.clone()).await;
            });
        })
    };

    let update_student = {
        let students = students.clone();
        let editing_student = editing_student.clone();

        Callback::from(move |updated_student: Student| {
            let students = students.clone();
            spawn_local(async move {
                if let Err(err) = Request::put(&format!(
                    "http://localhost:8000/students/{}",
                    updated_student.id.clone().unwrap().oid
                ))
                .json(&updated_student)
                .unwrap()
                .send()
                .await
                {
                    gloo::console::log!(format!("Error al actualizar estudiante: {:?}", err));
                }
                fetch_students(students).await;
            });

            editing_student.set(None);
        })
    };

    html! {
        <div>
            <h1>{ "Lista de Estudiantes" }</h1>
            <ul>
                { for students.borrow().iter().map(|student| html! {
                    <li>
                        <p>{ format!("{} - {} años - {}", student.name, student.age, student.subject) }</p>
                        <button
                            onclick={delete_student.reform({
                                let id = student.id.clone().unwrap().oid.clone();
                                move |_| id.clone()
                            })}
                        >
                            { "Eliminar" }
                        </button>
                        <button
                            onclick={Callback::from({
                                let editing_student = editing_student.clone();
                                let student = student.clone();
                                move |_| editing_student.set(Some(student))
                            })}
                        >
                            { "Editar" }
                        </button>
                    </li>
                }) }
            </ul>
            <div>
                <input
                    type="text"
                    placeholder="Nombre"
                    value={(*name).clone()}
                    oninput={Callback::from(move |e: InputEvent| {
                        name.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value());
                    })}
                />
                <input
                    type="text"
                    placeholder="Edad"
                    value={(*age).clone()}
                    oninput={Callback::from(move |e: InputEvent| {
                        age.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value());
                    })}
                />
                <input
                    type="text"
                    placeholder="Materia"
                    value={(*subject).clone()}
                    oninput={Callback::from(move |e: InputEvent| {
                        subject.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value());
                    })}
                />
                <button onclick={add_student}>{ "Agregar Estudiante" }</button>
            </div>
            {
                if let Some(student) = &*editing_student {
                    html! {
                        <div>
                            <h2>{ "Editar Estudiante" }</h2>
                            <input
                                type="text"
                                placeholder="Nombre"
                                value={student.name.clone()}
                                oninput={Callback::from({
                                    let editing_student = editing_student.clone();
                                    let student = student.clone();
                                    move |e: InputEvent| {
                                        let mut updated = student.clone();
                                        updated.name = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                                        editing_student.set(Some(updated));
                                    }
                                })}
                            />
                            <input
                                type="text"
                                placeholder="Edad"
                                value={student.age.to_string()}
                                oninput={Callback::from({
                                    let editing_student = editing_student.clone();
                                    let student = student.clone();
                                    move |e: InputEvent| {
                                        let mut updated = student.clone();
                                        updated.age = e.target_unchecked_into::<web_sys::HtmlInputElement>().value().parse().unwrap_or(0);
                                        editing_student.set(Some(updated));
                                    }
                                })}
                            />
                            <input
                                type="text"
                                placeholder="Materia"
                                value={student.subject.clone()}
                                oninput={Callback::from({
                                    let editing_student = editing_student.clone();
                                    let student = student.clone();
                                    move |e: InputEvent| {
                                        let mut updated = student.clone();
                                        updated.subject = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                                        editing_student.set(Some(updated));
                                    }
                                })}
                            />
                            <button onclick={update_student.reform({
                                let student = student.clone();
                                move |_| student.clone()
                            })}>{ "Actualizar" }</button>
                            <button onclick={Callback::from({
                                let editing_student = editing_student.clone();
                                move |_| editing_student.set(None)
                            })}>{ "Cancelar" }</button>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}

async fn fetch_students(students: UseStateHandle<Rc<RefCell<Vec<Student>>>>) {
    if let Ok(resp) = Request::get("http://localhost:8000/students").send().await {
        if let Ok(data) = resp.json::<Vec<Student>>().await {
            *students.borrow_mut() = data;
        }
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
