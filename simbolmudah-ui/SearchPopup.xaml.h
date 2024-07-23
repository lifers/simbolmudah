#pragma once

#include "SearchPopup.g.h"

namespace winrt::simbolmudah_ui::implementation
{
    struct SearchPopup : SearchPopupT<SearchPopup>
    {
        explicit SearchPopup(
            const LibSimbolMudah::KeyboardHook& hook,
            const LibSimbolMudah::SequenceDefinition& definition);
        SearchPopup(const SearchPopup&) = delete;
        SearchPopup& operator=(const SearchPopup&) = delete;

        Windows::Foundation::Collections::IObservableVector<simbolmudah_ui::SequenceDetail> SearchResults() const;

        void SearchBox_TextChanged(
            Microsoft::UI::Xaml::Controls::AutoSuggestBox const& sender,
            Microsoft::UI::Xaml::Controls::AutoSuggestBoxTextChangedEventArgs const& args);
        void SearchBox_SuggestionChosen(
            Microsoft::UI::Xaml::Controls::AutoSuggestBox const& sender, 
            Microsoft::UI::Xaml::Controls::AutoSuggestBoxSuggestionChosenEventArgs const& args) const;
        void Page_Loaded(IInspectable const& sender, Microsoft::UI::Xaml::RoutedEventArgs const& e);

    private:
        Windows::Foundation::IAsyncAction Search(hstring const& keyword);

        const apartment_context main_thread;
        const LibSimbolMudah::KeyboardHook hook;
        const LibSimbolMudah::SequenceDefinition sequenceDefinition;
        const Windows::Foundation::Collections::IObservableVector<simbolmudah_ui::SequenceDetail> searchResults;
        Windows::Foundation::IAsyncAction pendingSearch;
    };
}

namespace winrt::simbolmudah_ui::factory_implementation
{
    struct SearchPopup : SearchPopupT<SearchPopup, implementation::SearchPopup>
    {
    };
}
