export default class HandCard {
    constructor(scene) {
        this.render = (x, y, card_name,url) => {
            let sprite;
            let fetch=false;
            if (scene.textures.exists(card_name)){
                sprite=card_name;
            }else{
                sprite='card-back';
                fetch=true;
            }
            let card = scene.add.image(x, y, sprite).setScale(.5, .5).setInteractive();
            scene.input.setDraggable(card);
            if(fetch){
                scene.load.image(card_name,url);
                scene.load.once(Phaser.Loader.Events.COMPLETE, () => {
                    // texture loaded so use instead of the placeholder
                    card.setTexture(card_name)
                });
                scene.load.start();
            }
            return card;
        }
    }
}