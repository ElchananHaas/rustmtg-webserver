import Game from "./game.js";
var config = {
    type: Phaser.AUTO,
    scale: {
        mode: Phaser.Scale.RESIZE,
    },
    parent: 'phaser-game',
    antialias: true,
    antialiasGL:true,
    scene: [
        Game
    ]
};

var game = new Phaser.Game(config);
