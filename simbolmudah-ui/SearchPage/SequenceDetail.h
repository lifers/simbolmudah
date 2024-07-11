#pragma once
#include "SequenceDetail.g.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct SequenceDetail : SequenceDetailT<SequenceDetail>
    {
        SequenceDetail() = delete;
        SequenceDetail(SequenceDetail const&) = delete;
        SequenceDetail& operator=(SequenceDetail const&) = delete;
        SequenceDetail(
            Windows::Foundation::Collections::IVectorView<uint32_t> const& sequence,
            hstring const& result, hstring const& description);
        Windows::Foundation::Collections::IObservableVector<uint32_t> Sequence() const;
        hstring Result() const;
        hstring Description() const;

    private:
        const Windows::Foundation::Collections::IObservableVector<uint32_t> m_sequence;
        const hstring m_result;
		const hstring m_description;
    };
}
namespace winrt::simbolmudah_ui::factory_implementation
{
    struct SequenceDetail : SequenceDetailT<SequenceDetail, implementation::SequenceDetail>
    {
    };
}
