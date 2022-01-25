import Game from "./game.js";
var config = {
    type: Phaser.AUTO,
    scale: {
        mode: Phaser.Scale.RESIZE,
    },
    width: 1280,
    height: 780,
    parent: 'phaser-game',
    antialias: true,
    antialiasGL:true,
    scene: [
        Game
    ]
};

var game = new Phaser.Game(config);
