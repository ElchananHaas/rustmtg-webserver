const x_len=375;
const y_len=523;
const ul_x=-x_len/2;
const ul_y=-y_len/2;
export default class DispCard extends Phaser.GameObjects.Container{
    constructor(scene, x, y, card) {
        super(scene, x, y); 
        this.scene=scene;
        this.x_len=x_len;
        this.y_len=y_len;
        let card_background=null;
        let self=this;
        if(card==null){
            card_background = scene.add.image(0,0, "CardBack")
        }else{
            card_background = scene.add.image(0,0, "ArtifactCard")
        }
        card_background.setInteractive();
        this.add(card_background);
        this.attacking=[];
        this.card_background=card_background  
        card_background.on('pointerover', function () {
        });
        scene.input.setDraggable(card_background);
        card_background.on('drag', function (pointer, gameObject, dragX, dragY) {
            self.x = pointer.position.x;
            self.y = pointer.position.y;
        });
        this.setScale(.20,.20);
        if (card.tapped){
            this.rotation=.2;
        }
        else{
            this.rotation=0;
        } 
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
            let pt_backing=scene.add.image(-ul_x-10,-ul_y, "artifact_pt").setInteractive();
            this.add(pt_backing);
            let text=card.pt.power+"/"+card.pt.toughness;
            let line=this.scene.add.text(-ul_x-30,-ul_y-30,text);
            line.setAlign("center");
            line.setStyle({ fontFamily: 'gothic', fontSize: 40});
            line.setColor('#000000');
            this.add(line);
        }
        this.click_callback=function(pointer,gameObject){
        };
        this.card_background.on('pointerdown', function(pointer,gameObject){
            this.parentContainer.click_callback(pointer,gameObject);
        });
    }
    set_click_action_send(click_actions){
        this.click_callback=function(pointer,gameObject){
            if(click_actions.length==0){
                //Do nothing, this card has no actions
            }else if(click_actions.length==1){
                let to_send=JSON.stringify([click_actions[0]]);
                this.scene.socket.send(to_send);
                this.scene.clear_click_actions();
            }else{
                console.error("Selecting action dialog not implemented yet");
            }
        };
        this.addOutline(0xFFFF00);
    }
    set_click_action_attackers(targets,min,max){
        const scene=this.scene;
        console.log("settign click to attackers");
        for(const i in targets){
            const target=targets[i];
            if(target.Player){
                const player_box=scene.player_ui[target.Player].box_back;
                box_back.click_callback=function(pointer,gameObject){
                    console.log("clicked player now!");
                };
            }
        }
    }
    addOutline(color){
        this.outline=this.scene.add.rectangle(0,0,x_len,y_len+5)
        .setDepth(-10).setStrokeStyle(4,color);
        this.add(this.outline);
    }
    set_click_action_none(){
        this.click_callback=function(pointer,gameObject){
        };
        if(this.outline){
            this.outline.destroy();
            this.outline=null;
        }
    }
    add_text(y,text){
        let line=this.scene.add.text((ul_x+30),(ul_y+y),text);
        line.setStyle({ fontFamily: 'gothic', fontSize: 26});
        line.setColor('#000000');
        let max_width=(345-30);
        if(line.displayWidth>max_width ){
            line.setDisplaySize(max_width,25);
        }
        this.add(line);
    }
}