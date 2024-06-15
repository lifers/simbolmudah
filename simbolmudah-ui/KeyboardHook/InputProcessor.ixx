#include "pch.h"
#include <boost/container/static_vector.hpp>
export module InputProcessor;

import std.core;
import KeyboardTranslator;

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

export class InputProcessor
{
public:
	explicit InputProcessor(
		const winrt::delegate<winrt::fire_and_forget(std::wstring)>& reporterFn,
		KeyboardTranslator& translator
	) : m_reporterFn(reporterFn), m_keyboardTranslator(translator) {}
	~InputProcessor() = default;
	InputProcessor(const InputProcessor&) = delete;
	InputProcessor& operator=(const InputProcessor&) = delete;
	bool ProcessEvent(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage);

private:
	const winrt::delegate<winrt::fire_and_forget(std::wstring)> m_reporterFn;
	KeyboardTranslator& m_keyboardTranslator;
	boost::container::static_vector<INPUT, 16> m_inputBuffer;
	bool m_hasCapsLock{ (GetKeyState(VK_CAPITAL) & 1) != 0 };
	bool m_hasShift{ false };
	bool m_hasAltGr{ false };
	Stage m_stage{ Stage::Idle };
};
