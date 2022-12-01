cargo build
start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe port 4000
REM start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4010
REM start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4020
REM start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4030
REM start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4040
REM start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4050
REM start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4060
REM start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4070
REM start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4080
REM start d:\Unity\APTIVBotSimulation\Target_Windows\APTIVBotSimulation.exe -batchmode -nographics port 4090

REM timeout /t 10
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

