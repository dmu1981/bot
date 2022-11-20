cargo build
start "APTIV Bot (4444)" cmd /k target\debug\aptivbot.exe --simport 4444 
start "APTIV Bot (4445)" cmd /k target\debug\aptivbot.exe --simport 4445 --master false