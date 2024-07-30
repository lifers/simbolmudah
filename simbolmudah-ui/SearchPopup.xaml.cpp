#include "pch.hpp"
#include "SearchPopup.xaml.h"
#if __has_include("SearchPopup.g.cpp")
#include "SearchPopup.g.cpp"
#endif

using namespace winrt;
using namespace Microsoft::UI::Xaml;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
    using namespace LibSimbolMudah;
    using namespace Microsoft::UI::Xaml::Controls;
    using namespace Windows::Foundation;
    using namespace Collections;

    SearchPopup::SearchPopup(const KeyboardHook& hook, const SequenceDefinition& definition) :
        hook{ hook }, sequenceDefinition{ definition }, main_thread{ apartment_context() },
        searchResults{ single_threaded_observable_vector<simbolmudah_ui::SequenceDetail>() } {}

    IObservableVector<simbolmudah_ui::SequenceDetail> SearchPopup::SearchResults() const { return this->searchResults; }

    void SearchPopup::SearchBox_TextChanged(const AutoSuggestBox& sender, const AutoSuggestBoxTextChangedEventArgs&)
    {
        if (this->pendingSearch != nullptr)
        {
            this->pendingSearch.Cancel();
        }

        this->pendingSearch = this->Search(sender.Text());
        this->pendingSearch.Completed([weak_this{ this->get_weak() }](IAsyncAction const&, AsyncStatus const&)
        {
            if (auto strong_this{ weak_this.get() })
            {
                strong_this->pendingSearch = nullptr;
            }
        });
    }

    void SearchPopup::SearchBox_SuggestionChosen(const AutoSuggestBox&, const AutoSuggestBoxSuggestionChosenEventArgs&) const
    {
        this->hook.ResetStage();
    }

    void SearchPopup::Page_Loaded(const IInspectable&, const RoutedEventArgs&)
    {
        this->SearchBox().Focus(FocusState::Programmatic);
    }

    IAsyncAction SearchPopup::Search(const hstring& keyword)
    {
        const auto keyword_copy{ keyword };
        co_await resume_background();
        const auto results{ this->sequenceDefinition.Search(keyword_copy, 2000) };

        const auto size{ results.Size() };
        std::vector<SequenceDetail> toShow;
        toShow.reserve(size);

        for (const auto s: results)
        {
            toShow.emplace_back(s);
        }
        
        co_await main_thread;
        this->searchResults.ReplaceAll(toShow);
    }
}
