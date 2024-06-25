#include "pch.h"
#include <boost/container/static_vector.hpp>
#include <winrt/LibSimbolMudah.h>
export module KeyboardHook:InputDispatcher;

import std.core;
//import KeyboardTranslator;

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

export class InputDispatcher
{
public:
	explicit InputDispatcher(
		winrt::delegate<winrt::fire_and_forget(std::wstring)> const& reporterFn,
		winrt::LibSimbolMudah::KeyboardTranslator const& translator
	);
	~InputDispatcher();
	InputDispatcher(const InputDispatcher&) = delete;
	InputDispatcher& operator=(const InputDispatcher&) = delete;
	bool ProcessEvent(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage);
	winrt::fire_and_forget ResetStage(winrt::LibSimbolMudah::KeyboardTranslator const&, winrt::hstring const&);

private:
	const winrt::delegate<winrt::fire_and_forget(std::wstring)> m_reporterFn;
	const winrt::LibSimbolMudah::KeyboardTranslator& m_keyboardTranslator;
	winrt::event_token m_invalidToken;
	const winrt::apartment_context m_thread;
	boost::container::static_vector<INPUT, 16> m_inputBuffer;
	bool m_hasCapsLock{ (GetKeyState(VK_CAPITAL) & 1) != 0 };
	bool m_hasShift{ false };
	bool m_hasAltGr{ false };
	Stage m_stage{ Stage::Idle };
};
