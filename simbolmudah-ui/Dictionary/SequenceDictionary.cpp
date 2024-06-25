//module;
//#include "pch.h"
//#include <winrt/LibSimbolMudah.h>
//module KeyboardTranslator:SequenceDictionary;
//
//SequenceDictionary::SequenceDictionary()
//{
//	this->m_translator = winrt::LibSimbolMudah::SequenceTranslator();
//	this->m_translator.BuildDictionary();
//}
//
//SequenceDictionary::ResultVariant SequenceDictionary::AppendAndSearch(std::wstring sequence)
//{
//	this->m_sequence.append_range(sequence);
//	winrt::hstring tmp{ this->m_sequence };
//
//	try
//	{
//		winrt::hstring result { this->m_translator.Translate(tmp) };
//		return ValidString(result.c_str());
//	}
//	catch (winrt::hresult_error const& e)
//	{
//		if (e.code() == E_INVALIDARG)
//		{
//			if (this->m_sequence == L">")
//			{
//				return Incomplete();
//			}
//			else if (this->m_sequence == L"f")
//			{
//				return Incomplete();
//			}
//		}
//
//		return Invalid();
//	}
//}
//
//void SequenceDictionary::Clear() noexcept
//{
//	this->m_sequence.clear();
//}
