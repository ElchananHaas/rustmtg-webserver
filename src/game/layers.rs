use crate::game::*;


macro_rules! make_clear_components {
    ( $( $x:ty ),* ) => {
        fn clear_components(ents: &mut World){
            let entids=ents.iter().map(|entref| entref.entity() ).collect::<Vec<Entity>>();
            for id in entids{
            $(
                let _=ents.remove_one::<$x>(id);
            )*
            }
        }

    };
}
make_clear_components! {CardName,PT,Types,Supertypes,Vec<Ability>,HashSet<Subtype>,Controller}

impl Game{
    pub fn layers(&mut self){
        clear_components(&mut self.ents);
    }
}