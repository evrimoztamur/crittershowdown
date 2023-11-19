# TODO



## DONE

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