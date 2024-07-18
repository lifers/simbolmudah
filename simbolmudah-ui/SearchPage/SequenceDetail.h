#pragma once
#include "SequenceDetail.g.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct SequenceDetail : SequenceDetailT<SequenceDetail>
    {
        SequenceDetail() = delete;
        SequenceDetail(SequenceDetail const&) = delete;
        SequenceDetail& operator=(SequenceDetail const&) = delete;
        explicit SequenceDetail(
            Windows::Foundation::Collections::IVectorView<hstring> const& sequence,
            hstring const& result, hstring const& description, hstring const& codepoints);
        explicit SequenceDetail(LibSimbolMudah::SequenceDescription const& desc);
        Windows::Foundation::Collections::IObservableVector<hstring> Sequence() const;
        hstring Result() const;
        hstring Description() const;
        hstring Codepoints() const;

    private:
        const Windows::Foundation::Collections::IObservableVector<hstring> m_sequence;
        const hstring m_result;
		const hstring m_description;
        const hstring m_codepoints;
    };
}
namespace winrt::simbolmudah_ui::factory_implementation
{
    struct SequenceDetail : SequenceDetailT<SequenceDetail, implementation::SequenceDetail>
    {
    };
}
