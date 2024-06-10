#include "pch.h"
#include <boost/container/static_vector.hpp>
export module InputProcessor;

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
	explicit InputProcessor(const winrt::delegate<winrt::hstring>& reporterFn) : m_reporterFn(reporterFn) {}
	bool ProcessEvent(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage);

private:
	winrt::hstring StageToString() const;

	const winrt::delegate<winrt::hstring> m_reporterFn;
	boost::container::static_vector<INPUT, 16> m_inputBuffer;
	bool m_hasCapsLock{ (GetKeyState(VK_CAPITAL) & 1) != 0 };
	bool m_hasShift{ false };
	bool m_hasAltGr{ false };
	Stage m_stage{ Stage::Idle };
};
