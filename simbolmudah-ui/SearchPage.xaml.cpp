#include "pch.h"
#include "SearchPage.xaml.h"
#if __has_include("SearchPage.g.cpp")
#include "SearchPage.g.cpp"
#endif

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
	using namespace Windows::Foundation;
	using namespace Collections;
	using namespace Microsoft::UI::Xaml::Controls;

	simbolmudah_ui::SearchPageViewModel SearchPage::MainViewModel() const
	{
		return this->mainViewModel;
	}

	void SearchPage::SubmitSearch(AutoSuggestBox const& sender, AutoSuggestBoxTextChangedEventArgs const&)
	{
		if (this->currentSearch != nullptr)
		{
			this->currentSearch.Cancel();
		}
		this->currentSearch = this->mainViewModel.Search(sender.Text());
		this->currentSearch.Completed([this](IAsyncAction const&, AsyncStatus const&)
		{
			this->currentSearch = nullptr;
		});
	}
}
