import DispCard from "./card.js";
export default class Game extends Phaser.Scene {
    constructor() {
        super({
            key: 'Game'
        });
        this.PHASE_GREY=0x808080;
        this.ICON_SIZE=25;
        this.BOX_SIZE=125;
        this.BORDER_SIZE=5;
        this.CARD_WIDTH=100;
        this.CARD_HEIGHT=1.25*this.CARD_WIDTH;
    }
    
    preload() {
        this.self=this;
        this.canvas=this.sys.game.canvas;
        this.canvas.imageSmoothingEnabled = true;
        this.load.image('CardBack',"./cardback.svg");
        this.load.image('PlayerHand',"./hand.svg");
        this.load.image('heart',"./heart.svg");
        this.load.image('ArtifactCard',"./magic-m15-mse-style/acard.jpg")
        this.load.image('artifact_pt',"./magic-m15-mse-style/apt.png")
        this.load.image("White","./counters/w.svg");
        this.load.image("Blue","./counters/u.svg");
        this.load.image("Red","./counters/r.svg");
        this.load.image("Green","./counters/g.svg");
        this.load.image("Black","./counters/b.svg");
        this.load.image("Colorless","./counters/general.svg");
        this.load.image("Untap","./phases/untap.svg");
        this.load.image("Upkeep","./phases/upkeep.svg");
        this.load.image("Draw","./phases/draw.svg");
        this.load.image("FirstMain","./phases/main1.svg");
        this.load.image("BeginCombat","./phases/combat_start.svg");
        this.load.image("Attackers","./phases/combat_attackers.svg");
        this.load.image("Blockers","./phases/combat_blockers.svg");
        this.load.image("Damage","./phases/combat_damage.svg");
        this.load.image("EndCombat","./phases/combat_end.svg");
        this.load.image("SecondMain","./phases/main2.svg");
        this.load.image("EndStep","./phases/cleanup.svg");
        this.load.image("Pass","./phases/pass.svg");
        //this.load.setBaseURL('http://labs.phaser.io');
    }

    create() {
        let self=this;
        const socket = new WebSocket('ws://localhost:3030/gamesetup');
        socket.addEventListener('message', function (event) {
            let parsed=JSON.parse(event.data);
            console.log(parsed);
            if(parsed[0]==="GameState"){
                self.update_state(parsed);
            }   
            if(parsed[0]==="Action"){
                self.respond_action(parsed);
            }
            if(parsed[0]==="Attackers"){
                self.choose_attackers(parsed);
            } 
        });
        this.socket=socket;
        this.space_response="None";
        var spaceBar = this.input.keyboard.addKey(Phaser.Input.Keyboard.KeyCodes.SPACE);
        spaceBar.on('down', function(event){
            if(self.space_response=="send_empty"){
                self.socket.send("[]");
            }
        });
        this.disp_cards={};
        this.player_ui={};
        const phase_images_key=[
            "Untap",
            "Upkeep",
            "Draw",
            "FirstMain",
            "BeginCombat",
            "Attackers",
            "Blockers",
            "Damage",
            "EndCombat",
            "SecondMain",
            "EndStep",
        ];
        this.phase_images={}
        for(let i=0;i<phase_images_key.length;i++){
            const phase_image=phase_images_key[i];
            const image=this.add.image(this.ICON_SIZE,
                this.ICON_SIZE*(1+2*i),phase_image)
            .setScale(.8)
            .setTint(this.PHASE_GREY);
            this.phase_images[phase_image]=image;
        }
        const stack_start=2*this.ICON_SIZE+this.BOX_SIZE;
        this.add.rectangle(stack_start,0,this.CARD_WIDTH,this.height(),0x880000)
        .setDepth(-100).setOrigin(0,0).setStrokeStyle(1,0x000000);
        this.BATTLEFIELD_START=stack_start+this.CARD_WIDTH;
        this.add.rectangle(this.BATTLEFIELD_START,0,this.width()-this.BATTLEFIELD_START,this.height(),0x12093a)
        .setDepth(-1000).setOrigin(0,0);

    }

    width(){
        return this.canvas.width;
    }
    height(){
        return this.canvas.height;
    }
    phase_image_key(game_globals){
        const subphase=game_globals.subphase;
        if(subphase!=null){
            if(subphase=="FirstStrikeDamage"){
                return "Damage";
            }
            if(subphase=="Cleanup"){
                return "EndStep";
            }
            return subphase.toString();
        }
        if(game_globals.phase==null){
            return "Untap";
        }
        if(game_globals.phase=="Combat"){
            return "BeginCombat";
        }
        if(game_globals.phase=="Begin"){
            return "Untap";
        }
        if(game_globals.phase=="Ending"){
            return "EndStep";
        }
        return game_globals.phase.toString();
    }
    update_state(parsed){
        const state={
            me:parsed[1],
            ecs:parsed[2],
            players:parsed[3],
            globals:parsed[4],
        };
        const this_phase_key=this.phase_image_key(state.globals);
        for(const key in this.phase_images){
            this.phase_images[key].setTint(this.PHASE_GREY);
        }
        console.log(this_phase_key);
        this.phase_images[this_phase_key].setTint(0xFFFFFF);
        for (let card in this.disp_cards){
            this.disp_cards[card].destroy();
        }
        this.disp_cards={};
        let i=0;
        const num_players=Object.keys(state.players).length;
        for(const player in state.players){
            if(player!=state.me){    
                const bounds={
                    x:2*this.ICON_SIZE,
                    y:this.height()*i/num_players,
                    w:this.width()-2*this.ICON_SIZE,
                    h:this.height()/num_players
                }
                this.draw_player_board(player,bounds,state);
                i+=1;
            }
        }
        const bounds={
            x:2*this.ICON_SIZE,
            y:this.height()*(num_players-1)/num_players,
            w:this.width()-2*this.ICON_SIZE,
            h:this.height()/num_players
        }
        this.draw_player_board(state.me,bounds,state);
    }
    draw_player_board(player_id,bounds,state){
        const player=state.players[player_id];
        if(this.player_ui[player_id]!=null){
            for(const key in this.player_ui[player_id]){
                this.player_ui[player_id][key].destroy();
            }
        }
        this.player_ui[player_id]={};
        const ui=this.player_ui[player_id];
        let back_color=0x808080;
        if(state.globals.priority==player_id && player_id==state.me){
            back_color=0xA0A0A0;
        }
        ui.box_back = this.add.rectangle(bounds.x, bounds.y, this.BOX_SIZE, bounds.h, back_color)
        .setDepth(-10).setOrigin(0,0).setStrokeStyle(4,0x000000);
        
        ui.life_box= this.add.rectangle(bounds.x+this.BORDER_SIZE, 
            bounds.y+this.BORDER_SIZE,
             this.BOX_SIZE-2*this.BORDER_SIZE, 
             1.5*this.ICON_SIZE)
        .setDepth(-10).setOrigin(0,0).setStrokeStyle(2,0x000000);
        const heart_loc={
            x:bounds.x+this.BOX_SIZE/2,
            y:bounds.y+this.BORDER_SIZE+.75*this.ICON_SIZE
        };
        ui.heart_icon=this.add.image(heart_loc.x,heart_loc.y,"heart").setScale(.1);
        ui.life_text=this.add_text(heart_loc.x,heart_loc.y,player.life);
        const right_width=this.BOX_SIZE*3/4;
        const right_center=bounds.x+this.BOX_SIZE*5/8;
        const deck_y=heart_loc.y+this.ICON_SIZE*2;
        const hand_y=deck_y+right_width*.5;
        ui.deck=this.add.image(right_center,deck_y,"CardBack").setAngle(90)
        .setDisplaySize(right_width*.5, right_width*.8);
        this.draw_mana_circles(player_id,bounds,state);
        ui.deck_size=this.add_text(right_center,deck_y,player.library.length);
        ui.hand=this.add.image(right_center,hand_y,"PlayerHand").setScale(.1);
        ui.hand_back=this.add.circle(right_center,hand_y,this.ICON_SIZE/2,0xffffff).setDepth(1);
        ui.hand_size=this.add_text(right_center,hand_y,player.hand.length);
        const graveyard_y=hand_y+right_width*.5+2;
        ui.graveyard=this.add.rectangle(right_center,graveyard_y,right_width*.8, right_width*.5)
        .setStrokeStyle(2,0x000000);
        ui.graveyard_text=this.add_text(right_center,graveyard_y,player.graveyard.length);;
        const owned_exile=[];
        for(const card_id in state.globals.exile){
            const card=ecs[card_id];
            if(card.owner==player_id){
                owned_exile.push(card_id);
            }
        }
        const exile_y=graveyard_y+right_width*.5+10;
        ui.exile=this.add.rectangle(right_center,exile_y,right_width*.8, right_width*.5)
        .setStrokeStyle(2,0x000000);
        ui.exile_text=this.add_text(right_center,exile_y,owned_exile.length);;
        this.draw_hand(player_id,bounds,state);
        this.draw_player_battlefield(player_id,bounds,state);
    }
    draw_player_battlefield(player_id,bounds,state){
        const player=state.players[player_id];
        const ui=this.player_ui[player_id];
        const controlled=[];
        for(const i in state.globals.battlefield){
            const card_id=state.globals.battlefield[i];
            let card=state.ecs[card_id];
            if(card.controller==null){
                card.controller=card.owner;
            }
            if(card.controller==player_id){
                controlled.push(card_id);
            }
        }
        for(const i in controlled){
            this.add_disp_card(state.ecs,controlled[i],this.BATTLEFIELD_START+this.CARD_WIDTH/2
                + (Number(i) +.5) * this.CARD_WIDTH, bounds.y+this.CARD_HEIGHT/2);
        }
    
    }
    draw_hand(player_id,bounds,state){
        const player=state.players[player_id];
        const ui=this.player_ui[player_id];
        const hand_start=bounds.y+bounds.h-this.CARD_HEIGHT;
        ui.hand_background=this.add.rectangle(this.BATTLEFIELD_START,hand_start,
            this.width()-this.BATTLEFIELD_START,this.CARD_HEIGHT,0x008800).setOrigin(0,0).setDepth(-10);
        const hand=player.hand;
        for(const i in hand){
            this.add_disp_card(state.ecs,hand[i],this.BATTLEFIELD_START+this.CARD_WIDTH/2
                + (Number(i) +.5) * this.CARD_WIDTH, hand_start+this.CARD_HEIGHT/2);
            }
        }
    draw_mana_circles(player_id,bounds,state){
        const ui=this.player_ui[player_id];
        const colors=[
            "White",
            "Blue",
            "Black",
            "Red",
            "Green",
            "Colorless"
        ];
        const color_quantities={};
        for(const i in colors){
            color_quantities[colors[i]]=0;
        }
        const mana_pool=state.players[player_id].mana_pool;
        for(const i in mana_pool){
            const mana_id=mana_pool[i];
            const mana=state.globals.mana[mana_id];
            const color=mana.color;
            color_quantities[color]+=1;
        }
        const dot_size=this.BOX_SIZE/4;
        for(const i in colors){
            const color=colors[i];
            const loc={
                x:bounds.x+dot_size/2+5,
                y:bounds.y+this.BORDER_SIZE+1.5*this.ICON_SIZE+(Number(i)+.5)*1.1*dot_size,
            }
            const key_id=player_id+""+i;
            ui[key_id]=this.add.image(loc.x,loc.y,color).setDisplaySize(dot_size,dot_size);
            const quantity=color_quantities[color];
            ui[key_id+"text"]=this.add_text(loc.x,loc.y,quantity);
        }
        
    }
    add_text(x,y,text){
        return this.add.text(x,y,""+text)
        .setOrigin(0.5).setColor(0xFFFFFF).setDepth(10);
    }
    add_disp_card(ecs,index,x,y){
        let ent=ecs[index];
        let disp_card = new DispCard(this,x,y, ent);
        this.add.existing(disp_card);
        this.disp_cards[index]=disp_card;
        return disp_card;
    }
    choose_attackers(parsed){
        this.clear_click_actions();
        let pair=parsed[1].PairAB;
        if(pair.a.length==0){
            this.socket.send(JSON.stringify([]));
        }
    }
    respond_action(parsed){
        this.clear_click_actions();
        let choices=parsed[1].SelectN;
        let ents=choices.ents;
        this.action_ents=ents;
        if(ents.length==0){
            this.socket.send("[]");
        }else{
            for(let i=0;i<ents.length;i++){
                let ent=ents[i];
                if(ent.PlayLand!=null){
                    let disp_card=this.disp_cards[ent.PlayLand];
                    disp_card.set_click_action_send([i]);
                    disp_card.click_actions.push(i);
                }
                if(ent.ActivateAbility!=null){
                    let disp_card=this.disp_cards[ent.ActivateAbility.source];
                    disp_card.set_click_action_send([i]);
                }
                if(ent.Cast!=null){
                    let disp_card=this.disp_cards[ent.Cast.source_card];
                    disp_card.set_click_action_send([i]);
                }
            }
            this.space_response="send_empty"
        }
    }
    clear_click_actions(){
        for(let card in this.disp_cards){
            let disp_card=this.disp_cards[card];
            disp_card.set_click_action_none();
        }
        this.space_response="None"
    }
    
    update() {
    
    }
}