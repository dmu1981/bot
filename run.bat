cargo build
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe port 4000
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4010
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4020
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4030
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4040
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4050
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4060
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4070
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4080
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4090

timeout /t 10
start "APTIV Bot (4444)" cmd /k target\debug\aptivbot.exe --simport 4000
start "APTIV Bot (4444)" cmd /k target\debug\aptivbot.exe --simport 4010
start "APTIV Bot (4444)" cmd /k target\debug\aptivbot.exe --simport 4020
start "APTIV Bot (4444)" cmd /k target\debug\aptivbot.exe --simport 4030
start "APTIV Bot (4444)" cmd /k target\debug\aptivbot.exe --simport 4040
start "APTIV Bot (4444)" cmd /k target\debug\aptivbot.exe --simport 4050
start "APTIV Bot (4444)" cmd /k target\debug\aptivbot.exe --simport 4060
start "APTIV Bot (4444)" cmd /k target\debug\aptivbot.exe --simport 4070
start "APTIV Bot (4444)" cmd /k target\debug\aptivbot.exe --simport 4080
start "APTIV Bot (4444)" cmd /k target\debug\aptivbot.exe --simport 4090

