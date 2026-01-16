use marco_sparko::{ components::app::App};


fn main() -> anyhow::Result<()> {
    dioxus::launch(App);
    Ok(())
}