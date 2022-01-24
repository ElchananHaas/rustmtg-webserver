export default class HandCard extends Phaser.GameObjects.Container{
    constructor(scene, x, y, card) {
        super(scene, x, y); 
        this.scene=scene;
        const card_scale=.5;
        this.card_scale=card_scale;
        let card_background = scene.add.image(0,0, "artifact_card").setScale(card_scale,card_scale).setInteractive();
        this.add(card_background);
        if ("name" in card){
            this.add_text(25,card.name);
        }
        if ("subtypes" in card){
            this.add_text(294,card.subtypes.join(" "));
        }
    }
    add_text(y,text){
        let card_scale=this.card_scale;
        const x_len=375;
        const y_len=523;
        const ul_x=-x_len/2;
        const ul_y=-y_len/2;
        let line=this.scene.add.text((ul_x+30)*card_scale,(ul_y+y)*card_scale,text);
        line.setColor('#000000');
        let max_width=(345-30)*card_scale;
        if(line.displayWidth>max_width ){
            line.setDisplaySize(max_width,(25)*card_scale);
        }
        this.add(line);
    }
}
/*
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
                scene.load.image(url,url);
                scene.load.once(Phaser.Loader.Events.COMPLETE, () => {
                    // texture loaded so use instead of the placeholder
                    card.setTexture(url)
                });
                scene.load.start();
            }
            return card;
*/