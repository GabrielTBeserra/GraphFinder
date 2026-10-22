; ====================================================================
; GraphFinder NSIS Post-Install Hook
; Automatically registers GraphFinder CLI into Windows User PATH
; and creates graphfinder.cmd wrapper for seamless command line access
; ====================================================================

!macro NSIS_HOOK_POSTINSTALL
  DetailPrint "Configuring GraphFinder CLI in Windows User PATH..."

  ; 1. Create a graphfinder.cmd wrapper in $INSTDIR for lowercase cmd/powershell invocation
  FileOpen $0 "$INSTDIR\graphfinder.cmd" w
  FileWrite $0 "@echo off$\r$\n"
  FileWrite $0 "$\"%~dp0GraphFinder.exe$\" %*$\r$\n"
  FileClose $0

  ; 2. Add $INSTDIR to HKCU Environment PATH if not already present
  ReadRegStr $1 HKCU "Environment" "Path"
  StrCmp $1 "" empty_path check_path

empty_path:
  WriteRegExpandStr HKCU "Environment" "Path" "$INSTDIR"
  Goto notify_change

check_path:
  Push "$1"
  Push "$INSTDIR"
  Call StrStr
  Pop $2
  StrCmp $2 "" append_path
  Goto notify_change

append_path:
  WriteRegExpandStr HKCU "Environment" "Path" "$1;$INSTDIR"
  Goto notify_change

notify_change:
  ; Broadcast WM_SETTINGCHANGE (0x001A) to HWND_BROADCAST (0xFFFF)
  SendMessage 0xFFFF 0x001A 0 "STR:Environment" /TIMEOUT=5000
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Broadcast WM_SETTINGCHANGE on uninstall
  SendMessage 0xFFFF 0x001A 0 "STR:Environment" /TIMEOUT=5000
!macroend

; --- Helper Function: StrStr (Find substring in string) ---
!ifndef StrStr_FUNCTION
!define StrStr_FUNCTION
Function StrStr
  Exch $R1 ; substring
  Exch
  Exch $R2 ; string
  Push $R3
  Push $R4
  Push $R5

  StrLen $R3 $R1
  StrCpy $R4 0

loop:
  StrCpy $R5 $R2 $R3 $R4
  StrCmp $R5 $R1 found
  StrCmp $R5 "" not_found
  IntOp $R4 $R4 + 1
  Goto loop

found:
  StrCpy $R1 $R2 "" $R4
  Goto done

not_found:
  StrCpy $R1 ""

done:
  Pop $R5
  Pop $R4
  Pop $R3
  Pop $R2
  Exch $R1
FunctionEnd
!endif
