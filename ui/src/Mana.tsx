import React from "react";
import { GameState, PlayerView } from "./rustTypes";
const colors_map: { [key: string]: string } = {
    "White": "./counters/w.svg",
    "Blue": "./counters/u.svg",
    "Red": "./counters/r.svg",
    "Green": "./counters/g.svg",
    "Black": "./counters/b.svg",
    "Colorless": "./counters/general.svg",
};

type ManaSymbolProps = {
    color: string,
    quantity: number,
    height: number
}
function ManaSymbol(props: ManaSymbolProps) {
    return (<div className="custom-height-flexbox" style={{ height: props.height + "%" }}>
        <img src={colors_map[props.color]} className="full-height-image" alt=""></img>
        <div className="centering-div">
            <p className="text-over-content">
                {props.quantity}
            </p>
        </div>
    </div>);
}
export function ManaSymbols(props: {
    player: PlayerView,
    game: GameState
}) {
    const colors = [
        "White",
        "Blue",
        "Black",
        "Red",
        "Green",
        "Colorless"
    ];
    const color_quantities: { [key: string]: number } = {};
    colors.map(color => {
        color_quantities[color] = 0;
        return null;
    });
    const mana_pool = props.player.mana_pool;
    Object.keys(mana_pool).map(function(mana_id) {
        color_quantities[props.game.mana.ents[mana_id].color] += 1;
        return null;
    });
    return (
        <div className="mana-symbols">
            {colors.map(color =>
                <ManaSymbol key={color} color={color} quantity={color_quantities[color]} height={100 / colors.length} />
            )}
        </div>
    )
}