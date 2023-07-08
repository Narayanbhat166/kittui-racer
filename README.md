# A terminal type racer

Play typeracer with your friends at the comfort of your terminal.

## Server Architecture

<img width="1374" alt="Screenshot 2023-06-30 at 10 18 47 PM" src="https://github.com/Narayanbhat166/kittui-racer/assets/48803246/c6b07871-b136-4f49-8c20-4f7d3b0c405c">

## The process

- Client/Player opens the terminal application.
- Websocket connection in established.
  - Server sends `SuccessfulConnection` message with the user_name that was
    assigned
- Server sends an `UserStatus` message with the details of players who are
  currently online and their statuses ( if available or already in game).
- The Player can choose any of the online players who are available and challenge
  for a game by sending a `Challenge` message. Currently only one vs one is supported.
- The opponent can acccept the challenge by sending `AcceptChallenge`, if so, the game starts.

## Starting of the game

- Both parties are ready for the game.
- Server picks a random quote ( either from an api or from its database ), this quote is sent to both the users, with the countdown timer. `Ready(Text, start_time)`. It also spawns a task to handle the game after the countdown.
- After x time ( game start time ), server sends `GameStart` message.

## Communication when the game starts

- Each user will share his progress.
- The other user ( currently only one ), will get realtime update of the opponent.
- The game ends when any one of the user completes typing the whole message.
