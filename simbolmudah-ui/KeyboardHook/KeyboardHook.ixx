#include "pch.h"
export module KeyboardHook;

import Core;

using namespace winrt;
using namespace Windows::Foundation;

enum Stage : uint8_t
{
	Idle,
	ComposeKeydownFirst,
	ComposeKeyupFirst,
	ComposeKeydownSecond,
	SequenceMode,
	SearchMode,
	UnicodeMode
};

export class KeyboardHook
{
public:
	explicit KeyboardHook(const delegate<LowLevelKeyboardEvent>& reporterFn) : m_handle(RunAndMonitorListeners())
	{
		Reporter = reporterFn;
		m_hasCapsLock = GetKeyState(VK_CAPITAL) & 1;
	}
	~KeyboardHook();

private:
	static delegate<LowLevelKeyboardEvent> Reporter;
	static LRESULT CALLBACK KeyboardProcedure(int nCode, WPARAM wParam, LPARAM lParam);
	static bool IsHexadecimal(DWORD vkCode);
	IAsyncAction RunAndMonitorListeners();
	bool ProcessEvent(KBDLLHOOKSTRUCT keyInfo, WPARAM windowMessage);

	const IAsyncAction m_handle;
	HHOOK m_hook{ nullptr };
	bool m_hasCapsLock{ false };
	bool m_hasShift{ false };
	bool m_hasAltGr{ false };
	Stage m_stage{ Stage::Idle };
};
