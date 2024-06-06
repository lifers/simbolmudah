#include "pch.h"
export module Core;

export struct LowLevelKeyboardEvent
{
	DWORD vkCode;
	DWORD scanCode;
	DWORD flags;
	DWORD time;
	ULONG_PTR dwExtraInfo;
	WPARAM windowMessage;
};