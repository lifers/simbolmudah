#include "pch.hpp"
#include "SequenceDetail.h"
#include "SequenceDetail.g.cpp"

namespace
{
	using namespace winrt;
	using namespace Windows::Foundation::Collections;

	hstring ResultCodepointsToString(std::vector<UChar32> cps)
	{
		std::wstring result;

		for (auto cp: cps)
		{
			result.append(std::format(L"U+{:04X} ", cp));
		}

		return static_cast<hstring>(result);
	}

	std::vector<hstring> SequenceCodepointsToStrings(std::vector<UChar32> cps)
	{
		std::vector<hstring> result;
		UChar buffer[2] = {};
		UErrorCode status{ U_ZERO_ERROR };
		int32_t written{ 0 };

		for (auto cp: cps)
		{
			if (u_strFromUTF32(buffer, 2, &written, &cp, 1, &status) != &buffer[0])
			{
				throw hresult_error(E_FAIL, L"u_strFromUTF32 failed");
			}
			else if (U_FAILURE(status))
			{
				throw hresult_error(ERROR_NO_UNICODE_TRANSLATION, L"Failed to convert codepoint to UTF-16");
			}
			result.emplace_back(buffer, written);
		}

		return result;
	}

	std::vector<UChar32> ToCodepoints(hstring const& str)
	{
		std::vector<UChar32> codepoints;
		UCharIterator iter;
		uiter_setString(&iter, str.c_str(), str.size());

		while (iter.hasNext(&iter))
		{
			codepoints.push_back(uiter_next32(&iter));
		}

		return codepoints;
	}
}

namespace winrt::simbolmudah_ui::implementation
{
	using namespace Microsoft::UI::Xaml::Data;
	using namespace Windows::Foundation::Collections;

    SequenceDetail::SequenceDetail(
		IVectorView<hstring> const& sequence,
		hstring const& result, hstring const& description, hstring const& codepoints) :
			m_sequence(single_threaded_observable_vector<hstring>(std::vector<hstring>(sequence.begin(), sequence.end()))),
			m_result(result), m_description(description), m_codepoints(codepoints) {}

	SequenceDetail::SequenceDetail(LibSimbolMudah::SequenceDescription const& desc) :
		m_sequence(single_threaded_observable_vector<hstring>(SequenceCodepointsToStrings(ToCodepoints(desc.sequence)))),
		m_result(desc.result), m_description(desc.description), m_codepoints(ResultCodepointsToString(ToCodepoints(desc.result))) {}

	IObservableVector<hstring> SequenceDetail::Sequence() const { return this->m_sequence; }

    hstring SequenceDetail::Result() const { return this->m_result; }

    hstring SequenceDetail::Description() const { return this->m_description; }

	hstring SequenceDetail::Codepoints() const { return this->m_codepoints; }
}
