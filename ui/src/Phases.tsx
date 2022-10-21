import React from "react";

export const phase_image_map = {
    "Untap": "./phases/untap.svg",
    "Upkeep": "./phases/upkeep.svg",
    "Draw": "./phases/draw.svg",
    "FirstMain": "./phases/main1.svg",
    "BeginCombat": "./phases/combat_start.svg",
    "Attackers": "./phases/combat_attackers.svg",
    "Blockers": "./phases/combat_blockers.svg",
    "Damage": "./phases/combat_damage.svg",
    "EndCombat": "./phases/combat_end.svg",
    "SecondMain": "./phases/main2.svg",
    "EndStep": "./phases/cleanup.svg",
    "Pass": "./phases/nextturn.svg",
};
export function phase_image_key(phase?: string | null, subphase?: string | null): string {
    if (subphase) {
        if (subphase === "FirstStrikeDamage") {
            return "Damage";
        }
        if (subphase === "Cleanup") {
            return "EndStep";
        }
        return subphase.toString();
    }
    if (!phase) {
        return "Untap";
    }
    if (phase === "Combat") {
        return "BeginCombat";
    }
    if (phase === "Begin") {
        return "Untap";
    }
    if (phase === "Ending") {
        return "EndStep";
    }
    return phase.toString();
}
type PhaseImageProps = {
    height: number,
    src: string,
    name: string,
    darkened: boolean
};
function PhaseImage(props: PhaseImageProps) {
    return (<div className="full-size" style={{ height: props.height + "%" }} >
        <img
            src={props.src}
            alt={props.name}
            className="phase-image"></img>
        {props.darkened ? <div className="darken" style={{ height: props.height + "%" }}></div> : <div></div>}
    </div>);
}
type PhaseImagesProps = {
    phase?: string | null,
    subphase?: string | null,
    phase_image_map: { [key: string]: string },
};
export function PhaseImages(props: PhaseImagesProps) {
    const entries = Object.entries(props.phase_image_map);
    const current_phase = phase_image_key(props.phase, props.subphase);
    return (
        <div className="vertical-flexbox black-border">
            {entries.map(([key, url]) =>
                <PhaseImage
                    key={key}
                    src={url}
                    name={key}
                    height={100.0 / entries.length}
                    darkened={key !== current_phase}
                />
            )}
        </div>
    );
}