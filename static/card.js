const x_len=375;
const y_len=523;
const ul_x=-x_len/2;
const ul_y=-y_len/2;
export default class HandCard extends Phaser.GameObjects.Container{
    constructor(scene, x, y, card) {
        super(scene, x, y); 
        this.scene=scene;
        const card_scale=.5;
        this.x_len=x_len;
        this.y_len=y_len;
        this.card_scale=card_scale;
        let card_background=null;
        if(card==null){
            card_background = scene.add.image(0,0, "card_back")
        }else{
            card_background = scene.add.image(0,0, "artifact_card")
        }
        card_background.setScale(card_scale,card_scale).setInteractive();
        this.add(card_background);
        this.add_text(23,card.name);
        let type_line="";
        for (let t of card.supertypes){
            type_line+=(t+" ")
        }
        for (let t of card.types){
            type_line+=(t+" ")
        }
        type_line+="- "+card.subtypes.join(" ");
        this.add_text(292,type_line);
        if (card.pt!=null){
            let pt_backing=scene.add.image(-ul_x*(card_scale-.21),-ul_y*(card_scale-.05), "artifact_pt").setScale(card_scale,card_scale).setInteractive();
            this.add(pt_backing);
            let text=card.pt.power+"/"+card.pt.toughness;
            let line=this.scene.add.text(-ul_x*(card_scale-.26),-ul_y*(card_scale-.10),text);
            line.setAlign("center");
            line.setStyle({ fontFamily: 'gothic', fontSize: 20});
            line.setColor('#000000');
            this.add(line);
        }

        card_background.setInteractive();
        card_background.on('pointerover', function () {
        });
        scene.input.setDraggable(card_background);
        let t=this;
        card_background.on('drag', function (pointer, gameObject, dragX, dragY) {
            t.x = pointer.position.x;
            t.y = pointer.position.y;
        })
    }
    add_text(y,text){
        let card_scale=this.card_scale;
        let line=this.scene.add.text((ul_x+30)*card_scale,(ul_y+y)*card_scale,text);
        line.setStyle({ fontFamily: 'gothic', fontSize: 14});
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