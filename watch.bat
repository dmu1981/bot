cargo build

start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe port 5000

timeout /t 4
start "APTIV Bot Watcher (5000)" cmd /k target\debug\aptivbot.exe --simport 5000 watcher

