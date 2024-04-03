$windowTitle = "World in Conflict - InstallShield Wizard"

; read install dir from CLI args
$installDir = $CmdLine[1]

; next
WinWait($windowTitle)
WinActivate($windowTitle)
ConsoleWrite("next" & @CRLF)
ControlClick($windowTitle, "", "Button1")

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

Sleep(500)
WinWait($windowTitle)
WinActivate($windowTitle)
; select custom
;; click on list box
ConsoleWrite("activate listbox" & @CRLF)
ControlClick($windowTitle, "", "ListBox1")
Sleep(100)
;; send keyup
ConsoleWrite("send up" & @CRLF)
Send("{UP}")
;; click next
Sleep(100)
ConsoleWrite("next" & @CRLF)
ControlClick($windowTitle, "", "Button2")

Sleep(500)
WinWait($windowTitle)
WinActivate($windowTitle)
; click browse
ConsoleWrite("browse" & @CRLF)
ControlClick($windowTitle, "", "Button3")

; set directory
Sleep(500)
WinWait("Choose Folder")
WinActivate("Choose Folder")
ConsoleWrite("set install location" & @CRLF)
ControlSetText("Choose Folder", "", "[CLASS:Edit; INSTANCE:1]", $installDir)
Sleep(100)
; click ok
ControlClick("Choose Folder", "", "Button1")

; click next
Sleep(500)
WinWait($windowTitle)
WinActivate($windowTitle)
ConsoleWrite("next" & @CRLF)
ControlClick($windowTitle, "", "Button1")

; click next
Sleep(500)
WinWait($windowTitle)
WinActivate($windowTitle)
ConsoleWrite("next" & @CRLF)
ControlClick($windowTitle, "", "Button2")

; click install
Sleep(500)
WinWait($windowTitle)
WinActivate($windowTitle)
ConsoleWrite("install" & @CRLF)
ControlClick($windowTitle, "", "Button1")

While ControlGetText($windowTitle, "", "Static4") <> "InstallShield Wizard Complete"
    Sleep(500)
WEnd
Sleep(300)
ControlClick($windowTitle, "", "Button3")