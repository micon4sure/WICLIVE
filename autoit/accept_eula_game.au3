$windowTitle = "World in Conflict - InstallShield Wizard"

; wait for EULA window
WinWait($windowTitle)

; activate the window
$windowClass = "[CLASS:#32770]"
WinActivate($windowTitle)

; send ctrl end to eula window
ControlSend($windowTitle, "", "RichEdit20A1", "^{END}")

; send click to accept radio
ControlClick($windowTitle, "", "Button3")
; click next
ControlClick($windowTitle, "", "Button1")