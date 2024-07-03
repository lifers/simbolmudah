#include "pch.h"
#include "SearchPage.xaml.h"
#if __has_include("SearchPage.g.cpp")
#include "SearchPage.g.cpp"
#endif

using namespace winrt;
using namespace Microsoft::UI::Xaml;

// To learn more about WinUI, the WinUI project structure,
// and more about our project templates, see: http://aka.ms/winui-project-info.

namespace winrt::simbolmudah_ui::implementation
{
    int32_t SearchPage::MyProperty()
    {
        throw hresult_not_implemented();
    }

    void SearchPage::MyProperty(int32_t /* value */)
    {
        throw hresult_not_implemented();
    }

    void SearchPage::myButton_Click(IInspectable const&, RoutedEventArgs const&)
    {
        myButton().Content(box_value(L"Clicked"));
    }
}
