$windowTitle = "World in Conflict - InstallShield Wizard"

WinWait("World in Conflict")
WinActivate("World in Conflict")


$searchString = "This application will install"
While 1
    $controlText = ControlGetText("World in Conflict", "", "Static2")
    
    If StringInStr($controlText, $searchString) Then
        ExitLoop
    EndIf
    Sleep(500)
WEnd
ControlClick("World in Conflict", "", "Button1")


; accept eula
Sleep(500)
WinWait($windowTitle)
WinActivate($windowTitle)
; send click to accept radio
ConsoleWrite("activate radio accept" & @CRLF)
Sleep(300)
ControlClick($windowTitle, "", "Button3")
Sleep(300)
; click next
ConsoleWrite("next" & @CRLF)
ControlClick($windowTitle, "", "Button1")


While ControlGetText($windowTitle, "", "Static4") <> "The update was successfully installed."
    Sleep(500)
WEnd

Sleep(300)
ControlClick($windowTitle, "", "Button4")
