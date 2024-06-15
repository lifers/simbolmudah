module;
#include "pch.h"
module KeyboardTranslator;

namespace {

}

void KeyboardTranslator::TranslateAndForward(
	boost::container::static_vector<INPUT, 16> buffer, bool hasCapsLock, bool hasShift, bool hasAltGr, Destination destination
)
{
	// do something
}

KeyboardTranslator::StringVariant KeyboardTranslator::VKCodeToUnicode(
	uint32_t vkCode, uint32_t scanCode, const uint8_t (&keystate)[256], uint32_t flags
) const
{
	wchar_t buffer[8];
	if (int status = ToUnicodeEx(vkCode, scanCode, keystate, buffer, 8, flags, this->m_keyboardLayout); status > 0)
	{
		return LiveString{ std::wstring{ buffer, static_cast<size_t>(status) } };
	}
	else if (status < 0)
	{
		if (status = ToUnicodeEx(vkCode, scanCode, keystate, buffer, 8, flags, this->m_keyboardLayout); status > 0)
		{
			return DeadString{ std::wstring{ buffer, static_cast<size_t>(status) } };
		}
		else
		{
			return std::nullopt;
		}
	}
	else
	{
		return std::nullopt;
	}
}

void KeyboardTranslator::ToUnicodeExClearState() const
{
	uint8_t temp[256] = {};
	this->VKCodeToUnicode(VK_SPACE, 0, temp, 0);
	this->VKCodeToUnicode(VK_SPACE, 0, temp, 0);
}

void KeyboardTranslator::AnalyzeLayout()
{
	this->m_possibleAltGrKeys.clear();
	this->m_possibleDeadKeys.clear();

	std::wstring noAltGr[256];
	uint8_t keystate[256] = {};

	for (uint16_t i = 0; i < 0x400; ++i)
	{
		const uint16_t vkCode = i & 0xFF;
		const bool hasShift = (i & 0x100) != 0;
		const bool hasAltGr = (i & 0x200) != 0;

		keystate[VK_SHIFT] = hasShift ? 0x80 : 0;
		keystate[VK_MENU] = hasAltGr ? 0x80 : 0;
		keystate[VK_CONTROL] = hasAltGr ? 0x80 : 0;

		std::visit([this, &noAltGr, vkCode, hasAltGr](auto&& arg)
		{
			using T = std::decay_t<decltype(arg)>;
			if constexpr (!std::is_same_v<T, std::nullopt_t>)
			{
				if (hasAltGr)
				{
					this->m_possibleAltGrKeys.insert({ noAltGr[vkCode - 0x200], arg.data });
				}
				else
				{
					noAltGr[vkCode] = arg.data;
				}

				if constexpr (std::is_same_v<T, DeadString>)
				{
					this->m_possibleDeadKeys.insert({ arg.data, vkCode });
				}
			}
		}, this->VKCodeToUnicode(vkCode, 0, keystate, 0));
		
		this->ToUnicodeExClearState();
	}
}

void KeyboardTranslator::CheckLayoutAndUpdate()
{
	const auto foregroundWindow = GetForegroundWindow();
	const auto tid = GetWindowThreadProcessId(foregroundWindow, nullptr);
	const auto activeLayout = GetKeyboardLayout(tid);

	if (activeLayout != this->m_keyboardLayout)
	{
		this->m_keyboardLayout = activeLayout;
		this->AnalyzeLayout();
	}
}
