# TODO

- Mechanics

  - Collision events
    - Capture collision events
    - Turn collision events into `Collision(bug_index, bug_index)`
    - Process collision events per turn

  - Classes
    - Incorporate bug classes
    - Set physics properties based on class
      - Speed
      - Bounce
      - Mass
    - Collision events per class

  - Health
    - Add bug health that's different per class
    - Regenerate 1 health at turn end
    - Take off X health on collision
    - Disconnect bugs from the game after they pass
      - Draw them flipped over?

  - Game settings
    - Map size (22x22)
    - Game mode (KotH)

- Fixes
  - Bugs should have inherent direction besides `velocity.x.sign()`

- Polish
  - Particles
    - On collision
    - Various class-events
    - Win/Loss

  - Signals
    - Turn end
    - Scoring point and simulation stopping
    - Win/loss

## DONE

- Props
  - Obstacle bouncy mushrooms

- Mechanics
  - King of the Hill (KotH)
    - Draw circle in capture zone
    - Count bugs in zone
    - Add capture bar to bottom of the screen
    - Push time and capture bars to layer below bugs

- Refactoring Maginet codebase and strip it down to essentials
- Incorporating `rapier2d` as the physics backend
- Bugs
  - Including sprites
- Multiplayer
  - Lobby
    - Create new lobby
    - Join a lobby
    - Automatically join player to lobby
  - Game
    - Request turns
    - Synchronise turns between players
    - Execute a limited number of ticks per turn
    - Exhaust queue of turns
    - Do not process when no turns are available for processing
    - Fast-forward time without de-synchronisation issues
      - Target tick idea failed
      - Turn queue exhaustion worked
      - No counting necessary when everybody's synchronised
