module;
#include "pch.h"
module InputProcessor;

using namespace winrt;

namespace {
	constexpr INPUT KeyEventToInput(KBDLLHOOKSTRUCT keyEvent) noexcept
	{
		unsigned long dwFlags = KEYEVENTF_SCANCODE;
		if (keyEvent.flags & LLKHF_EXTENDED)
		{
			dwFlags |= KEYEVENTF_EXTENDEDKEY;
		}
		if (keyEvent.flags & LLKHF_UP)
		{
			dwFlags |= KEYEVENTF_KEYUP;
		}
		
		return INPUT{
			.type = INPUT_KEYBOARD,
			.ki = KEYBDINPUT{
				.wVk = static_cast<WORD>(keyEvent.vkCode),
				.wScan = static_cast<WORD>(keyEvent.scanCode),
				.dwFlags = dwFlags,
				.time = keyEvent.time,
				.dwExtraInfo = keyEvent.dwExtraInfo
			}
		};
	}

	constexpr std::wstring StageToString(Stage stage)
	{
		switch (stage)
		{
		case Idle: return L"Idle";
		case ComposeKeydownFirst: return L"ComposeKeydownFirst";
		case ComposeKeyupFirst: return L"ComposeKeyupFirst";
		case ComposeKeydownSecond: return L"ComposeKeydownSecond";
		case SequenceMode: return L"SequenceMode";
		case SearchMode: return L"SearchMode";
		case UnicodeMode: return L"UnicodeMode";
		default: std::unreachable();
		}
	}

	fire_and_forget SendBack(boost::container::static_vector<INPUT, 16> buffer, uint8_t start, uint8_t count)
	{
		co_await resume_background();
		if (SendInput(count, &buffer[start], sizeof(INPUT)) != count)
		{
			MessageBoxW(nullptr, L"Failed to send input properly", L"Error", MB_OK);
		}
	}
}

bool InputProcessor::ProcessEvent(KBDLLHOOKSTRUCT keyEvent, WPARAM windowMessage)
{
	const bool is_keydown = windowMessage == WM_KEYDOWN || windowMessage == WM_SYSKEYDOWN;

	// Update modifier key states
	switch (keyEvent.vkCode)
	{
	case VK_SHIFT: [[fallthrough]];
	case VK_LSHIFT: [[fallthrough]];
	case VK_RSHIFT:
		this->m_hasShift = is_keydown;
		return false;
	case VK_RMENU:
		this->m_hasAltGr = is_keydown;
		break;
	case VK_CAPITAL:
		if (is_keydown)
		{
			this->m_hasCapsLock = !this->m_hasCapsLock;
			return false;
		}
	}

	this->m_inputBuffer.emplace_back(KeyEventToInput(keyEvent));

	switch (this->m_stage)
	{
	case Idle:
		if (is_keydown && keyEvent.vkCode == VK_RMENU)
		{
			this->m_stage = ComposeKeydownFirst;
			this->m_reporterFn(StageToString(this->m_stage));
			return true;
		}
		else
		{
			this->m_inputBuffer.clear();
			this->m_reporterFn(StageToString(this->m_stage));
			return false;
		}
	case ComposeKeydownFirst:
		if (!is_keydown && keyEvent.vkCode == VK_RMENU)
		{
			this->m_stage = ComposeKeyupFirst;
		}
		else
		{
			SendBack(this->m_inputBuffer, 0, this->m_inputBuffer.size());
			this->m_inputBuffer.clear();
			this->m_stage = Idle;
		}
		this->m_reporterFn(StageToString(this->m_stage));
		return true;
	case ComposeKeyupFirst:
		if (is_keydown && keyEvent.vkCode == VK_RMENU)
		{
			this->m_stage = ComposeKeydownSecond;
		}
		else if (is_keydown && keyEvent.vkCode == 0x55) // VK_U
		{
			this->m_stage = UnicodeMode;
			this->m_keyboardTranslator.TranslateAndForward(
				this->m_inputBuffer, this->m_hasCapsLock, this->m_hasShift, this->m_hasAltGr,
				KeyboardTranslator::Destination::Unicode
			);
		}
		else
		{
			this->m_stage = SequenceMode;
			this->m_keyboardTranslator.TranslateAndForward(
				this->m_inputBuffer, this->m_hasCapsLock, this->m_hasShift, this->m_hasAltGr,
				KeyboardTranslator::Destination::Sequence
			);
		}
		this->m_reporterFn(StageToString(this->m_stage));
		return true;
	case ComposeKeydownSecond:
		if (!is_keydown && keyEvent.vkCode == VK_RMENU)
		{
			this->m_stage = SearchMode;
			this->m_inputBuffer.clear();
			// TODO: yield control to search UI
		}
		else
		{
			this->m_stage = SequenceMode;
			this->m_keyboardTranslator.TranslateAndForward(
				this->m_inputBuffer, this->m_hasCapsLock, this->m_hasShift, this->m_hasAltGr,
				KeyboardTranslator::Destination::Sequence
			);
		}
		this->m_reporterFn(StageToString(this->m_stage));
		return true;
	case SequenceMode:
		if (is_keydown)
		{
			this->m_keyboardTranslator.TranslateAndForward(
				this->m_inputBuffer, this->m_hasCapsLock, this->m_hasShift, this->m_hasAltGr,
				KeyboardTranslator::Destination::Sequence
			);
		}
		this->m_reporterFn(StageToString(this->m_stage));
		return true;
	case UnicodeMode:
		if (is_keydown)
		{
			this->m_keyboardTranslator.TranslateAndForward(
				this->m_inputBuffer, this->m_hasCapsLock, this->m_hasShift, this->m_hasAltGr,
				KeyboardTranslator::Destination::Unicode
			);
		}
		this->m_reporterFn(StageToString(this->m_stage));
		return true;
	default:
		std::unreachable();
	}
}
