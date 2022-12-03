cargo build


start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4010
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4020
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4030
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4040
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4050
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4060

timeout /t 7

start "APTIV Bot (4010)" cmd /k target\debug\aptivbot.exe --simport 4010
start "APTIV Bot (4020)" cmd /k target\debug\aptivbot.exe --simport 4020
start "APTIV Bot (4030)" cmd /k target\debug\aptivbot.exe --simport 4030
start "APTIV Bot (4040)" cmd /k target\debug\aptivbot.exe --simport 4040
start "APTIV Bot (4050)" cmd /k target\debug\aptivbot.exe --simport 4050
start "APTIV Bot (4060)" cmd /k target\debug\aptivbot.exe --simport 4060


timeout /t 3
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe port 4000

timeout /t 7
start "APTIV Bot (4000)" cmd /k target\debug\aptivbot.exe --simport 4000

