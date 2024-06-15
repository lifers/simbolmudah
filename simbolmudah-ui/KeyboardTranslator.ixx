#include "pch.h"
#include <boost/container/static_vector.hpp>
#include <boost/unordered/unordered_flat_map.hpp>
export module KeyboardTranslator;

import std.core;

export class KeyboardTranslator
{
public:
	KeyboardTranslator() = default;
	~KeyboardTranslator() = default;
	KeyboardTranslator(const KeyboardTranslator&) = delete;
	KeyboardTranslator& operator=(const KeyboardTranslator&) = delete;

	enum Destination : uint8_t
	{
		Sequence,
		Unicode
	};

	void TranslateAndForward(
		boost::container::static_vector<INPUT, 16> buffer,
		bool hasCapsLock, bool hasShift, bool hasAltGr, KeyboardTranslator::Destination destination
	);

private:
	struct LiveString { std::wstring data; };
	struct DeadString { std::wstring data; };
	typedef std::variant<LiveString, DeadString, std::nullopt_t> StringVariant;

	HKL m_keyboardLayout{ GetKeyboardLayout(0) };
	const winrt::Microsoft::UI::Dispatching::DispatcherQueueController m_controller{
		winrt::Microsoft::UI::Dispatching::DispatcherQueueController::CreateOnDedicatedThread()
	};
	
	// give at most 50% load factor
	boost::unordered_flat_map<std::wstring, std::wstring> m_possibleAltGrKeys{
		boost::unordered_flat_map<std::wstring, std::wstring>(2048)
	};
	boost::unordered_flat_map<std::wstring, uint16_t> m_possibleDeadKeys{
		boost::unordered_flat_map<std::wstring, uint16_t>(2048)
	};

	uint16_t m_savedDeadKey{ 0 };

	StringVariant VKCodeToUnicode(uint32_t vkCode, uint32_t scanCode, const uint8_t (&keystate)[256], uint32_t flags) const;
	void ToUnicodeExClearState() const;
	void AnalyzeLayout();
	void CheckLayoutAndUpdate();
};
