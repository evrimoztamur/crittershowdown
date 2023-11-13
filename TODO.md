# TODO

- Editor
  - Simulation results interface
  - Serde for level styles
    - Level styles in campaign menu..?
- Sound
  - Awaiting framk for SFX and music
  - State transitions
    - State requests background music
    - App handles transitions between pieces
    - Fade in-out
- Documentation UI
  - Table of contents
  - Text plus rendered image
    - Hardcode?

## DONE

- Deployment
  - Fix canvas sizing
  - Disable hotkeys for debugging on deploy version
  - Cropped demo ex. multiplayer and only 3-4 levels
  - Demo
    - Replace itch version
    - Replace evrim.zone version
    - Replace Steam version
- Arena
  - Pan/Move and lock onto levels
  - Previewing levels from list of codes/positions
  - Level names
  - Lock levels
  - Battle/Locked buttons
  - Select levels
  - Enter level and exit at same position back into Arena UI
  - Track wins
    - Record win upon exiting from won state
  - Lock level if no adjacent wins
  - Flipping mages when level won (+ medal from Mrmo)
  - 28 arena levels!
- Mechanics
  - Powerups
    - Diagonals
    - Plus-beam
    - Defensive mode
    - Rock as obstacle powerup
- Level/Editor
  - Select tileset
  - Fix menu state logic
  - Incorporate other boulder styles

## SKIP

- Tutorial
  - Hint that you can move diagonally for the final blow
  - Place the player into a 2v2 mini-battle for them to test their skills after they successfully finish the tutorial
  - Explain that the staff icon corresponds to the pattern
- Mechanics
  - Trial new mechanics
    - Board shenanigans
      - Force-shift all mages cardinal directions
      - Piece modifiers <https://nestorgames.com/rulebooks/ESSENTIA_EN.pdf>
    - Piece shenanigans
      - Promotion
        - On kill receive diagonals?
        - On low health turn defensive?
