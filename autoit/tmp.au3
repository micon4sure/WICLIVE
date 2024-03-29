$windowTitle = "World in Conflict - InstallShield Wizard"

; wait for EULA window
WinWait($windowTitle)

; activate the window
WinActivate($windowTitle)

; click on ListBox1
ControlClick($windowTitle, "", "ListBox1")
Sleep(300)
; send keyup
Send("{UP}")



;~ $cidList = "[CLASS:ListBox; INSTANCE:1]"

;~ $hdlWindow = WinGetHandle($windowTitle)
;~ $hdlList = ControlGetHandle($hdlWindow, "", $cidList)

;~ ;$list = ControlGetHandle($windowTitle, "", "ListBox1")
;~ ControlCommand($windowTitle, "", $hdlList, "SetCurrentSelection", 0) 