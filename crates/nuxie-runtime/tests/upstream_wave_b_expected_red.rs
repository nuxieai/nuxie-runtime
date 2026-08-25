//! Frozen Wave B ports of pinned C++ runtime TEST_CASEs 26-50.
//!
//! Every ignored Rust entry point embeds the complete upstream fixture, action
//! order, and assertion sequence. These remain expected-red until each body has
//! a typed Rust execution bridge; removing `#[ignore]` without translating the
//! embedded body is not a parity promotion.

fn pending_literal_port(pinned_cpp_case: &str) {
    assert!(pinned_cpp_case.starts_with("TEST_CASE("));
    assert!(pinned_cpp_case.contains('{') && pinned_cpp_case.ends_with('}'));
    panic!("expected-red: complete pinned case still awaits typed Rust execution");
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_converters_test case 1 awaits typed Rust execution"]
fn wave_b_data_binding_converters_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("list to length converter", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/list_to_length_test.riv", &silver);

    auto artboard = file->artboardDefault();

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    auto list = vmi->propertyValue("lis");
    auto childVM = file->viewModel("child");
    int cnt = 0;
    while (cnt++ < 4)
    {
        silver.addFrame();
        auto childVMI = file->createDefaultViewModelInstance(childVM);
        if (childVMI != nullptr)
        {
            auto listItem = make_rcp<ViewModelInstanceListItem>();
            listItem->viewModelInstance(childVMI);
            list->as<ViewModelInstanceList>()->addItem(listItem);
        }
        // first advance to set view model value
        stateMachine->advanceAndApply(0.1f);
        // second advance to measure
        stateMachine->advanceAndApply(0.1f);

        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("list_to_length_test"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_converters_test case 2 awaits typed Rust execution"]
fn wave_b_data_binding_converters_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("data converter interpolator resets on binding", "[silver]")
{
    SerializingFactory silver;
    auto file =
        ReadRiveFile("assets/data_converter_interpolator_reset.riv", &silver);

    auto artboard = file->artboardDefault();

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    auto renderer = silver.makeRenderer();
    int viewModelId = artboard.get()->viewModelId();
    {
        auto vmi = viewModelId == -1
                       ? file->createViewModelInstance(artboard.get())
                       : file->createViewModelInstance(viewModelId, 0);
        auto numProp =
            vmi->propertyValue("xPos")->as<ViewModelInstanceNumber>();
        numProp->propertyValue(250);
        auto colProp = vmi->propertyValue("col")->as<ViewModelInstanceColor>();
        auto redColor = (255 << 24) | (255 << 16);
        colProp->propertyValue(redColor);

        stateMachine->bindViewModelInstance(vmi);
        stateMachine->advanceAndApply(0.1f);

        artboard->draw(renderer.get());

        auto greenColor = (255 << 24) | (255 << 8);
        colProp->propertyValue(greenColor);
        numProp->propertyValue(500);

        int frames = (int)(1.0f / 0.016f);
        for (int i = 0; i < frames; i++)
        {
            silver.addFrame();
            stateMachine->advanceAndApply(0.016f);
            artboard->draw(renderer.get());
        }
    }
    // When a new binding is applied, interpolators are reset and the initial
    // value is not interpolated
    {
        silver.addFrame();
        auto vmi = viewModelId == -1
                       ? file->createViewModelInstance(artboard.get())
                       : file->createViewModelInstance(viewModelId, 0);
        auto numProp =
            vmi->propertyValue("xPos")->as<ViewModelInstanceNumber>();
        numProp->propertyValue(250);
        auto colProp = vmi->propertyValue("col")->as<ViewModelInstanceColor>();
        auto redColor = (255 << 24) | (255 << 16);
        colProp->propertyValue(redColor);
        stateMachine->bindViewModelInstance(vmi);
        stateMachine->advanceAndApply(0.1f);

        artboard->draw(renderer.get());

        auto blueColor = (255 << 24) | 255;
        colProp->propertyValue(blueColor);
        numProp->propertyValue(0);

        int frames = (int)(1.0f / 0.016f);
        for (int i = 0; i < frames; i++)
        {
            silver.addFrame();
            stateMachine->advanceAndApply(0.016f);
            artboard->draw(renderer.get());
        }
    }

    CHECK(silver.matches("data_converter_interpolator_reset"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_converters_test case 3 awaits typed Rust execution"]
fn wave_b_data_binding_converters_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Interpolations that change duration to zero work correctly",
          "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/interpolation_zero_duration.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);
    auto objectX = vmi->propertyValue("objectX")->as<ViewModelInstanceNumber>();
    auto interpValue =
        vmi->propertyValue("interpValue")->as<ViewModelInstanceNumber>();

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);
    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    objectX->propertyValue(200);

    int frames = (int)(1.5f / 0.1f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
    }

    interpValue->propertyValue(0);
    stateMachine->advanceAndApply(0.016f);
    objectX->propertyValue(400);
    stateMachine->advanceAndApply(0.016f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
    }

    interpValue->propertyValue(1);
    stateMachine->advanceAndApply(0.016f);
    objectX->propertyValue(200);
    stateMachine->advanceAndApply(0.016f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("interpolation_zero_duration"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_cycle_test case 1 awaits typed Rust execution"]
fn wave_b_data_binding_cycle_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("view model changed by child updates on the parent on next frame",
          "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test_3.riv");

    auto artboard = file->artboard("main-1")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto stateMachineInstance = artboard->defaultStateMachine();
    stateMachineInstance->bindViewModelInstance(viewModelInstance);
    REQUIRE(stateMachineInstance != nullptr);
    stateMachineInstance->advanceAndApply(0.0f);
    REQUIRE(artboard->find<rive::Rectangle>("sized-rect-path") != nullptr);
    auto rect = artboard->find<rive::Rectangle>("sized-rect-path");
    REQUIRE(rect->width() == 100.0f);
    // This click event is captured by a child nested artboard that updates a
    // view model value
    stateMachineInstance->pointerDown(rive::Vec2D(75.0f, 75.0f));
    stateMachineInstance->pointerUp(rive::Vec2D(75.0f, 75.0f));
    stateMachineInstance->advanceAndApply(0.0f);
    // A single advance is needed to reflect the changes on the parent affected
    // by that view model value
    REQUIRE(rect->width() == 200.0f);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_cycle_test case 2 awaits typed Rust execution"]
fn wave_b_data_binding_cycle_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("view model changed by parent updates on the child on next frame",
          "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test_3.riv");

    auto artboard = file->artboard("main-2")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto stateMachineInstance = artboard->defaultStateMachine();
    stateMachineInstance->bindViewModelInstance(viewModelInstance);
    REQUIRE(stateMachineInstance != nullptr);
    stateMachineInstance->advanceAndApply(0.0f);
    REQUIRE(artboard->find<rive::NestedArtboard>("child-2") != nullptr);
    auto nestedArtboardChild = artboard->find<rive::NestedArtboard>("child-2");

    auto nestedArtboardArtboardChild = nestedArtboardChild->artboardInstance();
    REQUIRE(nestedArtboardArtboardChild != nullptr);
    auto rect =
        nestedArtboardArtboardChild->find<rive::Rectangle>("child-rect-path");
    REQUIRE(rect != nullptr);
    REQUIRE(rect->width() == 100.0f);

    stateMachineInstance->pointerDown(rive::Vec2D(250.0f, 250.0f));
    stateMachineInstance->pointerUp(rive::Vec2D(250.0f, 250.0f));
    stateMachineInstance->advanceAndApply(0.0f);
    // // A single advance is needed to reflect the changes on the child
    // affected
    // // by that view model value
    REQUIRE(rect->width() == 200.0f);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_cycle_test case 3 awaits typed Rust execution"]
fn wave_b_data_binding_cycle_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE(
    "view model changed by child event updates on the parent on next frame",
    "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test_3.riv");

    auto artboard = file->artboard("main-3")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto stateMachineInstance = artboard->defaultStateMachine();
    stateMachineInstance->bindViewModelInstance(viewModelInstance);
    REQUIRE(stateMachineInstance != nullptr);
    stateMachineInstance->advanceAndApply(0.0f);
    REQUIRE(artboard->find<rive::Rectangle>("sized-rect-path") != nullptr);
    auto rect = artboard->find<rive::Rectangle>("sized-rect-path");
    REQUIRE(rect->width() == 100.0f);
    // An event is triggered at 0.5s that will perform a change on the view
    // model
    stateMachineInstance->advanceAndApply(0.5f);
    REQUIRE(rect->width() == 100.0f);
    // An extra advance is needed for the change to be propagated
    stateMachineInstance->advanceAndApply(0.0f);
    REQUIRE(rect->width() == 200.0f);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_cycle_test case 4 awaits typed Rust execution"]
fn wave_b_data_binding_cycle_test_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE(
    "view model changed by parent event updates on the child on next frame",
    "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test_3.riv");

    auto artboard = file->artboard("main-4")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto stateMachineInstance = artboard->defaultStateMachine();
    stateMachineInstance->bindViewModelInstance(viewModelInstance);
    REQUIRE(stateMachineInstance != nullptr);
    stateMachineInstance->advanceAndApply(0.0f);
    REQUIRE(artboard->find<rive::NestedArtboard>("child-4") != nullptr);
    auto nestedArtboardChild = artboard->find<rive::NestedArtboard>("child-4");

    auto nestedArtboardArtboardChild = nestedArtboardChild->artboardInstance();
    REQUIRE(nestedArtboardArtboardChild != nullptr);
    auto rect =
        nestedArtboardArtboardChild->find<rive::Rectangle>("child-rect-path");
    REQUIRE(rect != nullptr);
    REQUIRE(rect->width() == 100.0f);
    // An event on the parent triggers a view model change
    stateMachineInstance->advanceAndApply(0.5f);
    REQUIRE(rect->width() == 100.0f);
    // An extra advance is needed for the change to be propagated
    stateMachineInstance->advanceAndApply(0.0f);
    REQUIRE(rect->width() == 200.0f);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_cycle_test case 5 awaits typed Rust execution"]
fn wave_b_data_binding_cycle_test_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("view model changed by child target to source prop changes on the "
          "same frame on parent",
          "[data binding]")
{

    auto file = ReadRiveFile("assets/data_binding_test_3.riv");

    auto artboard = file->artboard("main-5")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto stateMachineInstance = artboard->defaultStateMachine();
    stateMachineInstance->bindViewModelInstance(viewModelInstance);
    REQUIRE(stateMachineInstance != nullptr);
    stateMachineInstance->advanceAndApply(0.0f);
    REQUIRE(artboard->find<rive::TextValueRun>("text-run-test") != nullptr);
    auto textRunChild = artboard->find<rive::TextValueRun>("text-run-test");
    REQUIRE(textRunChild->text() == "before");
    stateMachineInstance->advanceAndApply(0.5f);
    REQUIRE(textRunChild->text() == "after");
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_cycle_test case 6 awaits typed Rust execution"]
fn wave_b_data_binding_cycle_test_006_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("view model changed by parent target to source prop changes on the "
          "same frame on child",
          "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test_3.riv");

    auto artboard = file->artboard("main-6")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto stateMachineInstance = artboard->defaultStateMachine();
    stateMachineInstance->bindViewModelInstance(viewModelInstance);
    REQUIRE(stateMachineInstance != nullptr);
    stateMachineInstance->advanceAndApply(0.0f);
    REQUIRE(artboard->find<rive::NestedArtboard>("child-6") != nullptr);
    auto nestedArtboardChild = artboard->find<rive::NestedArtboard>("child-6");

    auto nestedArtboardArtboardChild = nestedArtboardChild->artboardInstance();
    REQUIRE(nestedArtboardArtboardChild != nullptr);
    auto textRunParent =
        nestedArtboardArtboardChild->find<rive::TextValueRun>("child-text-run");
    REQUIRE(textRunParent != nullptr);
    REQUIRE(textRunParent->text() == "parent-before");
    stateMachineInstance->advanceAndApply(0.5f);
    REQUIRE(textRunParent->text() == "parent-after");
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_cycle_test case 7 awaits typed Rust execution"]
fn wave_b_data_binding_cycle_test_007_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("view model changed by three artboard levels", "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test_3.riv");

    auto artboard = file->artboard("main-7")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto stateMachineInstance = artboard->defaultStateMachine();
    stateMachineInstance->bindViewModelInstance(viewModelInstance);
    REQUIRE(stateMachineInstance != nullptr);
    stateMachineInstance->advanceAndApply(0.0f);
    REQUIRE(artboard->find<rive::TextValueRun>("main-run") != nullptr);
    auto mainRun = artboard->find<rive::TextValueRun>("main-run");
    REQUIRE(artboard->find<rive::NestedArtboard>("child-7") != nullptr);
    auto nestedArtboardChild = artboard->find<rive::NestedArtboard>("child-7");

    auto nestedArtboardArtboardChild = nestedArtboardChild->artboardInstance();
    REQUIRE(nestedArtboardArtboardChild != nullptr);
    auto childRun =
        nestedArtboardArtboardChild->find<rive::TextValueRun>("child-run");
    REQUIRE(childRun != nullptr);
    REQUIRE(nestedArtboardArtboardChild->find<rive::NestedArtboard>(
                "grand-child-7") != nullptr);
    auto nestedArtboardGrandChild =
        nestedArtboardArtboardChild->find<rive::NestedArtboard>(
            "grand-child-7");
    auto nestedArtboardGrandArtboardChild =
        nestedArtboardGrandChild->artboardInstance();
    auto grandChildRun =
        nestedArtboardGrandArtboardChild->find<rive::TextValueRun>(
            "grand-child-run");
    REQUIRE(grandChildRun != nullptr);

    stateMachineInstance->advanceAndApply(0.5f);
    REQUIRE(mainRun->text() == "main-test-2");
    REQUIRE(childRun->text() == "main-test-2");
    REQUIRE(grandChildRun->text() == "main-test-2");

    stateMachineInstance->advanceAndApply(1.5f);
    REQUIRE(mainRun->text() == "child-text-1");
    REQUIRE(childRun->text() == "child-text-1");
    REQUIRE(grandChildRun->text() == "child-text-1");

    stateMachineInstance->advanceAndApply(0.5f);
    REQUIRE(mainRun->text() == "child-text-2");
    REQUIRE(childRun->text() == "child-text-2");
    REQUIRE(grandChildRun->text() == "child-text-2");

    stateMachineInstance->advanceAndApply(1.5f);
    REQUIRE(mainRun->text() == "grand-child-text-1");
    REQUIRE(childRun->text() == "grand-child-text-1");
    REQUIRE(grandChildRun->text() == "grand-child-text-1");

    stateMachineInstance->advanceAndApply(.5f);
    REQUIRE(mainRun->text() == "grand-child-text-2");
    REQUIRE(childRun->text() == "grand-child-text-2");
    REQUIRE(grandChildRun->text() == "grand-child-text-2");
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_fonts_test case 1 awaits typed Rust execution"]
fn wave_b_data_binding_fonts_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Data bind font", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/data_bind_font_test.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(vmi != nullptr);
    auto renderer = silver.makeRenderer();
    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    silver.addFrame();

    // Load the kablammo ttf through the file's factory and set it as the value
    // of the bound ViewModelInstanceAssetFont. This should reshape the text
    // with the new font on the next advance/draw.
    auto fontBytes = ReadFile("assets/kablammo.ttf");
    auto font = silver.decodeFont(fontBytes);
    REQUIRE(font != nullptr);

    auto fontProperty = vmi->propertyValue("fontProperty");
    REQUIRE(fontProperty != nullptr);
    REQUIRE(fontProperty->is<rive::ViewModelInstanceAssetFont>());
    fontProperty->as<rive::ViewModelInstanceAssetFont>()->value(font.get());

    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Click in a square that fires a listener
    // that sets the font to another font value.
    stateMachine->pointerDown(rive::Vec2D(490, 490));
    stateMachine->pointerUp(rive::Vec2D(490, 490));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    silver.addFrame();
    // Click in a square that fires a listener
    // that sets the font to another font property.
    stateMachine->pointerDown(rive::Vec2D(490, 20));
    stateMachine->pointerUp(rive::Vec2D(490, 20));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    CHECK(silver.matches("data_bind_font_test"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_fonts_test case 2 awaits typed Rust execution"]
fn wave_b_data_binding_fonts_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Font data bind stores and clears the font on the property",
          "[data binding]")
{
    auto file = ReadRiveFile("assets/data_bind_font_test.riv");

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);
    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(vmi != nullptr);
    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);

    auto property = vmi->propertyValue("fontProperty");
    REQUIRE(property != nullptr);
    REQUIRE(property->is<rive::ViewModelInstanceAssetFont>());
    auto fontProperty = property->as<rive::ViewModelInstanceAssetFont>();
    REQUIRE(fontProperty->asset() != nullptr);

    // Assigning a decoded font stores it on the property's backing FontAsset.
    auto fontBytes = ReadFile("assets/kablammo.ttf");
    auto font = HBFont::Decode(fontBytes);
    REQUIRE(font != nullptr);
    fontProperty->value(font.get());
    stateMachine->advanceAndApply(0.0f);
    CHECK(fontProperty->asset()->font().get() == font.get());

    // Swapping to a different font updates the backing FontAsset.
    auto font2Bytes = ReadFile("assets/nabla.ttf");
    auto font2 = HBFont::Decode(font2Bytes);
    REQUIRE(font2 != nullptr);
    fontProperty->value(font2.get());
    stateMachine->advanceAndApply(0.0f);
    CHECK(fontProperty->asset()->font().get() == font2.get());

    // Passing null clears the backing font.
    fontProperty->value(nullptr);
    stateMachine->advanceAndApply(0.0f);
    CHECK(fontProperty->asset()->font() == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_images_test case 1 awaits typed Rust execution"]
fn wave_b_data_binding_images_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Test data binding images from file assets", "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_images_test.riv");

    auto artboard = file->artboard("main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);
    artboard->advance(0.0f);

    // View model properties
    // Images
    auto mainProperty = viewModelInstance->propertyValue("main_im");
    REQUIRE(mainProperty != nullptr);
    REQUIRE(mainProperty->is<rive::ViewModelInstanceAssetImage>());
    auto mainImgProperty =
        mainProperty->as<rive::ViewModelInstanceAssetImage>();
    auto sub1VMIProperty = viewModelInstance->propertyValue("sub_1");
    REQUIRE(sub1VMIProperty != nullptr);
    REQUIRE(sub1VMIProperty->is<rive::ViewModelInstanceViewModel>());
    auto referencedViewModel =
        sub1VMIProperty->as<rive::ViewModelInstanceViewModel>()
            ->referenceViewModelInstance();
    auto sub1Property = referencedViewModel->propertyValue("sub_1_im");
    REQUIRE(sub1Property != nullptr);
    REQUIRE(sub1Property->is<rive::ViewModelInstanceAssetImage>());
    auto sub1ImgProperty =
        sub1Property->as<rive::ViewModelInstanceAssetImage>();

    // File assets
    auto assets = file->assets();
    // Image layers

    REQUIRE(artboard->find<rive::Image>("root_img") != nullptr);
    auto rootImage = artboard->find<rive::Image>("root_img");

    REQUIRE(artboard->find<rive::NestedArtboard>("sub_1") != nullptr);
    auto nestedArtboardSub1 = artboard->find<rive::NestedArtboard>("sub_1");

    auto nestedArtboardArtboardSub1 = nestedArtboardSub1->artboardInstance();
    REQUIRE(nestedArtboardArtboardSub1 != nullptr);
    REQUIRE(nestedArtboardArtboardSub1->find<rive::Image>("sub_1_img") !=
            nullptr);
    auto sub1Image = nestedArtboardArtboardSub1->find<rive::Image>("sub_1_img")
                         ->as<rive::Image>();
    REQUIRE(sub1Image != nullptr);
    // Validations
    // Ensure view model image asset is the same as the image's image asset
    auto mainAsset = assets[mainImgProperty->propertyValue()].get();
    auto imageAsset = rootImage->imageAsset();
    REQUIRE(imageAsset == mainAsset);
    auto sub1Asset = assets[sub1ImgProperty->propertyValue()].get();
    auto sub1ImageAsset = sub1Image->imageAsset();
    REQUIRE(sub1Asset == sub1ImageAsset);
    // Change values
    mainImgProperty->propertyValue(2);
    sub1ImgProperty->propertyValue(6);
    artboard->advance(0.0f);
    auto updatedMainAsset = assets[mainImgProperty->propertyValue()].get();
    auto updatedSub1Asset = assets[sub1ImgProperty->propertyValue()].get();
    // Ensure image is no longer the same
    REQUIRE(imageAsset != updatedMainAsset);
    REQUIRE(sub1ImageAsset != updatedSub1Asset);
    // Ensure new image asset is the one assigned to the view model property
    // asset
    imageAsset = rootImage->imageAsset();
    REQUIRE(imageAsset == updatedMainAsset);
    sub1ImageAsset = sub1Image->imageAsset();
    REQUIRE(sub1ImageAsset == updatedSub1Asset);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_images_test case 2 awaits typed Rust execution"]
fn wave_b_data_binding_images_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Embedded images can be reset by passing null", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/viewmodel_image_reset.riv", &silver);

    auto artboard = file->artboardDefault();

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);
    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    auto vmiImage =
        vmi->propertyValue("img")->as<ViewModelInstanceAssetImage>();
    vmiImage->value(nullptr);
    silver.addFrame();
    stateMachine->advanceAndApply(0.1f);
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("viewmodel_image_reset"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_images_test case 3 awaits typed Rust execution"]
fn wave_b_data_binding_images_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Image based conditions work", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/viewmodel_based_condition.riv", &silver);

    auto artboard = file->artboardDefault();

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);
    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->advanceAndApply(1.1f);
    // Since these tests are relying on an event, we need to advance ne extra
    // time for the event to be processed
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->advanceAndApply(1.1f);
    // Since these tests are relying on an event, we need to advance ne extra
    // time for the event to be processed
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("viewmodel_based_condition"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_images_test case 4 awaits typed Rust execution"]
fn wave_b_data_binding_images_test_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Dynamic image binding with listener action", "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/image_binding_with_listener.riv", &silver);

    auto artboard = file->artboardNamed("main");

    silver.frameSize(artboard->width(), artboard->height());

    auto renderer = silver.makeRenderer();

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi = file->createViewModelInstance(artboard.get()->viewModelId(), 0);
    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(650.0f, 650.0f));
    stateMachine->pointerUp(rive::Vec2D(650.0f, 650.0f));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    auto imageFile = ReadFile("assets/open_source.jpg");
    REQUIRE(imageFile.size() == 8880);

    auto decodedImage = silver.decodeImage(imageFile);
    auto imgProp =
        vmi->propertyValue("image1")->as<rive::ViewModelInstanceAssetImage>();
    imgProp->value(decodedImage.get());

    silver.addFrame();
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(650.0f, 650.0f));
    stateMachine->pointerUp(rive::Vec2D(650.0f, 650.0f));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    imgProp->value(nullptr);

    silver.addFrame();
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(650.0f, 650.0f));
    stateMachine->pointerUp(rive::Vec2D(650.0f, 650.0f));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("image_binding_with_listener"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_images_test case 5 awaits typed Rust execution"]
fn wave_b_data_binding_images_test_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Image fit & alignment with databound images", "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/image_fit_alignment.riv", &silver);

    auto artboard = file->artboardNamed("Main");
    REQUIRE(artboard != nullptr);
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    // Coverage for generated ImageBase fit/alignment setters/getters.
    auto firstImage = [&]() -> rive::Image* {
        for (auto image : artboard->objects<rive::Image>())
        {
            return image;
        }
        return nullptr;
    };
    auto imageForCoverage = firstImage();
    REQUIRE(imageForCoverage != nullptr);
    auto originalFit = imageForCoverage->fit();
    auto originalAlignmentX = imageForCoverage->alignmentX();
    auto originalAlignmentY = imageForCoverage->alignmentY();

    auto testFit = originalFit == static_cast<uint32_t>(rive::Fit::contain)
                       ? static_cast<uint32_t>(rive::Fit::cover)
                       : static_cast<uint32_t>(rive::Fit::contain);
    auto testAlignmentX = originalAlignmentX == -1.0f ? 1.0f : -1.0f;
    auto testAlignmentY = originalAlignmentY == -1.0f ? 1.0f : -1.0f;

    imageForCoverage->fit(testFit);
    imageForCoverage->alignmentX(testAlignmentX);
    imageForCoverage->alignmentY(testAlignmentY);

    CHECK(imageForCoverage->fit() == testFit);
    CHECK(imageForCoverage->alignmentX() == testAlignmentX);
    CHECK(imageForCoverage->alignmentY() == testAlignmentY);

    imageForCoverage->fit(originalFit);
    imageForCoverage->alignmentX(originalAlignmentX);
    imageForCoverage->alignmentY(originalAlignmentY);

    CHECK(imageForCoverage->fit() == originalFit);
    CHECK(imageForCoverage->alignmentX() == originalAlignmentX);
    CHECK(imageForCoverage->alignmentY() == originalAlignmentY);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    auto imageProperty = vmi->propertyValue("imageProperty");
    REQUIRE(imageProperty != nullptr);
    REQUIRE(imageProperty->is<rive::ViewModelInstanceAssetImage>());
    auto imageAssetProperty =
        imageProperty->as<rive::ViewModelInstanceAssetImage>();
    auto noScaleImage = [&]() -> rive::Image* {
        for (auto image : artboard->objects<rive::Image>())
        {
            if (image->fit() == static_cast<uint32_t>(rive::Fit::none))
            {
                return image;
            }
        }
        return nullptr;
    };

    auto assets = file->assets();
    auto findAssetIndexByName = [&](const char* assetName) -> size_t {
        for (size_t i = 0; i < assets.size(); i++)
        {
            if (assets[i]->name() == assetName)
            {
                return i;
            }
        }
        return assets.size();
    };

    auto image1Index = findAssetIndexByName("image1");
    auto image2Index = findAssetIndexByName("image2");
    auto image3Index = findAssetIndexByName("image3");
    REQUIRE(image1Index != assets.size());
    REQUIRE(image2Index != assets.size());
    REQUIRE(image3Index != assets.size());

    int frames = 20;
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    imageAssetProperty->propertyValue(static_cast<uint32_t>(image2Index));
    stateMachine->advanceAndApply(0.0f);
    auto noScale = noScaleImage();
    REQUIRE(noScale != nullptr);
    CHECK(noScale->transform()[4] < 0.0f);
    CHECK(noScale->transform()[5] < 0.0f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    imageAssetProperty->propertyValue(static_cast<uint32_t>(image3Index));
    stateMachine->advanceAndApply(0.0f);
    noScale = noScaleImage();
    REQUIRE(noScale != nullptr);
    CHECK(noScale->transform()[4] < 0.0f);
    CHECK(noScale->transform()[5] < 0.0f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("image_fit_alignment"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_images_test case 6 awaits typed Rust execution"]
fn wave_b_data_binding_images_test_006_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Image fit & alignment with databound images 2", "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/image_fit_alignment_2.riv", &silver);

    auto artboard = file->artboardNamed("Main");
    REQUIRE(artboard != nullptr);
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = 60;
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("image_fit_alignment_2"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_images_test case 7 awaits typed Rust execution"]
fn wave_b_data_binding_images_test_007_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Image fit & alignment images 3", "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/image_fit_alignment_3.riv", &silver);

    auto artboard = file->artboardNamed("Artboard");
    REQUIRE(artboard != nullptr);
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = 60;
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("image_fit_alignment_3"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_images_test case 8 awaits typed Rust execution"]
fn wave_b_data_binding_images_test_008_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Image fit & alignment with image scaling", "[silver]")
{
    rive::SerializingFactory silver;
    auto file =
        ReadRiveFile("assets/image_fit_alignment_updated_test.riv", &silver);

    auto artboard = file->artboardNamed("Main");
    REQUIRE(artboard != nullptr);
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = 60;
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("image_fit_alignment_updated_test"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_images_test case 9 awaits typed Rust execution"]
fn wave_b_data_binding_images_test_009_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Layout image composes user scale on top of fit for 7.2 files",
          "[assets]")
{
    auto legacyBytes = ReadFile("assets/image_fit_alignment.riv");
    REQUIRE(legacyBytes.size() > 6);
    REQUIRE(legacyBytes[0] == 'R');
    REQUIRE(legacyBytes[1] == 'I');
    REQUIRE(legacyBytes[2] == 'V');
    REQUIRE(legacyBytes[3] == 'E');
    // Major/minor are single-byte varuints here.
    REQUIRE(legacyBytes[4] == 7);
    REQUIRE(legacyBytes[5] < 2);

    auto modernBytes = legacyBytes;
    modernBytes[5] = 2; // bump the minor version to 7.2

    auto xAxisScale = [](const rive::Mat2D& m) {
        return rive::Vec2D(m[0], m[1]).length();
    };

    // Loads the file, binds its view model, and holds the file/artboard/state
    // machine alive plus the layout images in artboard order (identical between
    // the two files since they share bytes).
    struct Loaded
    {
        rive::rcp<rive::File> file;
        std::unique_ptr<rive::ArtboardInstance> artboard;
        std::unique_ptr<rive::StateMachineInstance> stateMachine;
        std::vector<rive::Image*> images;
    };
    auto load = [](std::vector<uint8_t>& bytes,
                   rive::Factory* factory) -> Loaded {
        Loaded loaded;
        rive::ImportResult result;
        loaded.file = rive::File::import(bytes, factory, &result);
        REQUIRE(result == rive::ImportResult::success);
        loaded.artboard = loaded.file->artboardNamed("Main");
        REQUIRE(loaded.artboard != nullptr);
        loaded.stateMachine = loaded.artboard->stateMachineAt(0);
        REQUIRE(loaded.stateMachine != nullptr);
        int viewModelId = loaded.artboard->viewModelId();
        auto vmi =
            viewModelId == -1
                ? loaded.file->createViewModelInstance(loaded.artboard.get())
                : loaded.file->createViewModelInstance(viewModelId, 0);
        loaded.stateMachine->bindViewModelInstance(vmi);
        loaded.stateMachine->advanceAndApply(0.1f);
        for (auto image : loaded.artboard->objects<rive::Image>())
        {
            if (image->parent() != nullptr &&
                image->parent()->is<rive::LayoutComponent>())
            {
                loaded.images.push_back(image);
            }
        }
        return loaded;
    };

    // Use SerializingFactory so the in-band images actually decode (it gives
    // real image dimensions, which the fit depends on). One per file, kept
    // alive for the file's lifetime.
    rive::SerializingFactory legacyFactory;
    rive::SerializingFactory modernFactory;
    auto legacy = load(legacyBytes, &legacyFactory);
    auto modern = load(modernBytes, &modernFactory);
    REQUIRE(!legacy.images.empty());
    REQUIRE(legacy.images.size() == modern.images.size());

    // The layout is animated and may start collapsed (fit ~ 0). Advance both
    // files in lockstep (identical state) until a layout image is open, then
    // pick that image. This avoids depending on async decode timing landing on
    // an open frame.
    rive::NoOpRenderer renderer;
    size_t pick = legacy.images.size();
    for (int frame = 0; frame < 120 && pick == legacy.images.size(); frame++)
    {
        for (size_t i = 0; i < legacy.images.size(); i++)
        {
            if (xAxisScale(legacy.images[i]->worldTransform()) > 1.0f)
            {
                pick = i;
                break;
            }
        }
        if (pick != legacy.images.size())
        {
            break;
        }
        legacy.stateMachine->advanceAndApply(0.016f);
        modern.stateMachine->advanceAndApply(0.016f);
    }
    REQUIRE(pick != legacy.images.size());

    auto legacyImage = legacy.images[pick];
    auto modernImage = modern.images[pick];

    // 7.2 keeps the stored (non-default) user scale; legacy overwrites it with
    // the fit.
    float userScaleX = modernImage->scaleX();
    REQUIRE(userScaleX != Approx(1.0f));
    CHECK(legacyImage->scaleX() != Approx(userScaleX));

    // Modern renders at fit * userScale; legacy at fit only (same layout
    // state).
    float legacyScale = xAxisScale(legacyImage->worldTransform());
    float modernScale = xAxisScale(modernImage->worldTransform());
    REQUIRE(legacyScale > 0.0f);
    CHECK(modernScale == Approx(legacyScale * userScaleX));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_images_test case 10 awaits typed Rust execution"]
fn wave_b_data_binding_images_test_010_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Stateful component image bind", "[silver]")
{
    rive::SerializingFactory silver;
    auto file =
        ReadRiveFile("assets/stateful_component_image_test.riv", &silver);

    auto artboard = file->artboardDefault();

    silver.frameSize(artboard->width(), artboard->height());

    auto renderer = silver.makeRenderer();

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi = file->createViewModelInstance(artboard.get()->viewModelId(), 0);
    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    auto imageFile = ReadFile("assets/open_source.jpg");
    REQUIRE(imageFile.size() == 8880);

    auto decodedImage = silver.decodeImage(imageFile);
    auto imgProp =
        vmi->propertyValue("img")->as<rive::ViewModelInstanceAssetImage>();
    imgProp->value(decodedImage.get());

    silver.addFrame();
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("stateful_component_image_test"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_keyframes case 1 awaits typed Rust execution"]
fn wave_b_data_binding_keyframes_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Data binding keyframes", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/data_bind_keyframes_test.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    auto renderer = silver.makeRenderer();

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    int frames = (int)(1.0f / 0.2f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.2f);
        artboard->draw(renderer.get());
    }

    auto keyfTextStartProp =
        vmi->propertyValue("keyfTextStart")->as<ViewModelInstanceString>();
    keyfTextStartProp->propertyValue("updated--text");

    auto colorStartProp =
        vmi->propertyValue("colorStart")->as<ViewModelInstanceColor>();
    auto yellowColor = (255 << 24) | (255 << 16) | (255 << 8);
    colorStartProp->propertyValue(yellowColor);

    auto startXProp =
        vmi->propertyValue("startX")->as<ViewModelInstanceNumber>();
    startXProp->propertyValue(100);

    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.2f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("data_bind_keyframes_test"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_keyframes case 2 awaits typed Rust execution"]
fn wave_b_data_binding_keyframes_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("keyframe value binds resolve view-model values on the first frame",
          "[data binding]")
{
    auto file = ReadRiveFile("assets/data_bind_keyframes_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    auto sm = artboard->stateMachineAt(0);
    REQUIRE(sm != nullptr);
    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(vmi != nullptr);

    // Distinctive sentinels set before binding, so the very first applied frame
    // must reflect them (holders are primed when the data bind is added).
    setStartText(vmi, "SENTINEL_START");
    setStartX(vmi, 424242.0f);

    sm->bindViewModelInstance(vmi);
    sm->advanceAndApply(0.0f);

    auto* run = firstTextRun(artboard.get());
    REQUIRE(run != nullptr);
    CHECK(run->text() == "SENTINEL_START");        // KeyFrameString bind
    CHECK(anyNodeHasX(artboard.get(), 424242.0f)); // KeyFrameDouble bind
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_keyframes case 3 awaits typed Rust execution"]
fn wave_b_data_binding_keyframes_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("keyframe value binds update when the source view-model changes",
          "[data binding]")
{
    auto file = ReadRiveFile("assets/data_bind_keyframes_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    auto sm = artboard->stateMachineAt(0);
    auto vmi = file->createDefaultViewModelInstance(artboard.get());

    setStartText(vmi, "first");
    setStartX(vmi, 10.0f);
    sm->bindViewModelInstance(vmi);
    sm->advanceAndApply(0.0f);

    auto* run = firstTextRun(artboard.get());
    REQUIRE(run != nullptr);
    REQUIRE(run->text() == "first");
    REQUIRE(anyNodeHasX(artboard.get(), 10.0f));

    // Change the sources; dt=0 keeps the playhead on the start keyframe, so the
    // exact new bound values must land after the next advance.
    setStartText(vmi, "second");
    setStartX(vmi, 987.0f);
    sm->advanceAndApply(0.0f);

    CHECK(run->text() == "second");
    CHECK(anyNodeHasX(artboard.get(), 987.0f));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_keyframes case 4 awaits typed Rust execution"]
fn wave_b_data_binding_keyframes_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("keyframe interpolation reads the data-bound start value",
          "[data binding]")
{
    auto file = ReadRiveFile("assets/data_bind_keyframes_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    auto sm = artboard->stateMachineAt(0);
    auto vmi = file->createDefaultViewModelInstance(artboard.get());

    // A large, distinctive start value the authored keyframe would never
    // produce. The node tweens from this bound start toward its authored end,
    // so once it's into the tween its x must be strictly between the two,
    // proving the interpolation's from-endpoint is the bound value (not the
    // authored one).
    const float boundStart = 100000.0f;
    setStartX(vmi, boundStart);
    sm->bindViewModelInstance(vmi);

    sm->advanceAndApply(0.0f);
    REQUIRE(anyNodeHasX(artboard.get(), boundStart)); // start endpoint applied

    sm->advanceAndApply(0.5f); // well into the tween
    bool inTween = false;
    for (auto* node : artboard->find<Node>())
    {
        // Moved off the bound start but still dominated by it => interpolating
        // from the bound value.
        if (node->x() > 50000.0f && node->x() < boundStart)
        {
            inTween = true;
            break;
        }
    }
    CHECK(inTween);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_keyframes case 5 awaits typed Rust execution"]
fn wave_b_data_binding_keyframes_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("standalone animation instance ignores keyframe value binds",
          "[data binding]")
{
    auto file = ReadRiveFile("assets/data_bind_keyframes_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    REQUIRE(artboard->animationCount() > 0);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    setStartText(vmi, "SHOULD_NOT_BIND");
    setStartX(vmi, 424242.0f);
    // Bind the artboard's data context but drive playback via a *standalone*
    // LinearAnimationInstance (not a state machine). Keyframe value binds are
    // built only for state-machine-driven instances, so the authored keyframe
    // values must apply here instead of the bound ones.
    artboard->bindViewModelInstance(vmi);

    auto animation = artboard->animationAt(0);
    REQUIRE(animation != nullptr);
    animation->advanceAndApply(0.0f);

    auto* run = firstTextRun(artboard.get());
    REQUIRE(run != nullptr);
    CHECK(run->text() != "SHOULD_NOT_BIND");
    CHECK_FALSE(anyNodeHasX(artboard.get(), 424242.0f));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 1 awaits typed Rust execution"]
fn wave_b_data_binding_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("artboard with bound properties", "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test.riv");

    auto artboard = file->artboard("artboard-1")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);
    artboard->advance(0.0f);
    REQUIRE(artboard->find<rive::Rectangle>("bound_rect") != nullptr);
    auto rectMapped = artboard->find<rive::Rectangle>("bound_rect");
    REQUIRE(rectMapped->width() == 100.0f);
    REQUIRE(artboard->find<rive::Shape>("bound_rect_shape") != nullptr);
    auto shapeMapped = artboard->find<rive::Shape>("bound_rect_shape");
    // Rotation has a system converter applied to it
    REQUIRE(shapeMapped->rotation() == Approx(1.5708f));

    REQUIRE(shapeMapped->children()[1]->is<rive::Fill>());
    rive::Fill* rectMappadFill = shapeMapped->children()[1]->as<rive::Fill>();
    REQUIRE(rectMappadFill->paint()->is<rive::SolidColor>());
    REQUIRE(rectMappadFill->paint()->as<rive::SolidColor>()->colorValue() ==
            rive::colorARGB(255, 255, 0, 0));

    REQUIRE(artboard->find<rive::TextValueRun>("bound_text_run") != nullptr);
    auto textRunMapped = artboard->find<rive::TextValueRun>("bound_text_run");
    REQUIRE(textRunMapped->text() == "bound text");

    REQUIRE(artboard->find<rive::FollowPathConstraint>("") != nullptr);
    auto followPathConstraint = artboard->find<rive::FollowPathConstraint>("");
    REQUIRE(followPathConstraint->orient() == false);

    // View model properties
    // Number
    auto widthProperty = viewModelInstance->propertyValue("width");
    REQUIRE(widthProperty != nullptr);
    REQUIRE(widthProperty->is<rive::ViewModelInstanceNumber>());
    // Number with comverter
    auto rotationProperty = viewModelInstance->propertyValue("rotation");
    REQUIRE(rotationProperty != nullptr);
    REQUIRE(rotationProperty->is<rive::ViewModelInstanceNumber>());
    // Color
    auto colorProperty = viewModelInstance->propertyValue("color");
    REQUIRE(colorProperty != nullptr);
    REQUIRE(colorProperty->is<rive::ViewModelInstanceColor>());
    // String
    auto textProperty = viewModelInstance->propertyValue("text");
    REQUIRE(textProperty != nullptr);
    REQUIRE(textProperty->is<rive::ViewModelInstanceString>());
    // Boolean
    auto orientProperty = viewModelInstance->propertyValue("orient");
    REQUIRE(orientProperty != nullptr);
    REQUIRE(orientProperty->is<rive::ViewModelInstanceBoolean>());
    // Update view model values
    widthProperty->as<rive::ViewModelInstanceNumber>()->propertyValue(200.0f);
    rotationProperty->as<rive::ViewModelInstanceNumber>()->propertyValue(
        180.0f);
    colorProperty->as<rive::ViewModelInstanceColor>()->propertyValue(
        rive::colorARGB(255, 0, 255, 0));
    textProperty->as<rive::ViewModelInstanceString>()->propertyValue(
        "New text");
    orientProperty->as<rive::ViewModelInstanceBoolean>()->propertyValue(true);
    // Advance artboard so data binds apply
    artboard->advance(0.0f);
    // Validate new properties
    REQUIRE(rectMapped->width() == 200.0f);
    REQUIRE(shapeMapped->rotation() == Approx(3.14159f));
    REQUIRE(rectMappadFill->paint()->as<rive::SolidColor>()->colorValue() ==
            rive::colorARGB(255, 0, 255, 0));
    REQUIRE(textRunMapped->text() == "New text");
    REQUIRE(followPathConstraint->orient() == true);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 2 awaits typed Rust execution"]
fn wave_b_data_binding_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("state machine led by enums and triggers", "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test.riv");

    auto artboard = file->artboard("artboard-2")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto machine = artboard->defaultStateMachine();
    machine->bindViewModelInstance(viewModelInstance);
    REQUIRE(machine != nullptr);
    //

    REQUIRE(artboard->find<rive::Shape>("color_rectangle") != nullptr);
    auto shapeMapped = artboard->find<rive::Shape>("color_rectangle");

    REQUIRE(shapeMapped->children()[1]->is<rive::Fill>());
    REQUIRE(shapeMapped->x() == 250);
    REQUIRE(shapeMapped->y() == 250);
    rive::Fill* rectMappadFill = shapeMapped->children()[1]->as<rive::Fill>();
    REQUIRE(rectMappadFill->paint()->is<rive::SolidColor>());
    REQUIRE(rectMappadFill->paint()->as<rive::SolidColor>()->colorValue() ==
            rive::colorARGB(255, 116, 116, 116));

    // View model properties
    // Enum
    auto stateProperty = viewModelInstance->propertyValue("state");
    REQUIRE(stateProperty != nullptr);
    REQUIRE(stateProperty->is<rive::ViewModelInstanceEnum>());
    // Trigger
    auto triggerProperty = viewModelInstance->propertyValue("trigger-prop");
    REQUIRE(triggerProperty != nullptr);
    REQUIRE(triggerProperty->is<rive::ViewModelInstanceTrigger>());
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(rectMappadFill->paint()->as<rive::SolidColor>()->colorValue() ==
            rive::colorARGB(255, 255, 0, 0));
    // Update view model properties
    // Update enum by index
    stateProperty->as<rive::ViewModelInstanceEnum>()->value(1);

    // Advance state machine
    machine->advanceAndApply(0.0f);
    // Validate values have updated
    REQUIRE(rectMappadFill->paint()->as<rive::SolidColor>()->colorValue() ==
            rive::colorARGB(255, 0, 255, 0));
    REQUIRE(shapeMapped->x() == 150);
    REQUIRE(shapeMapped->y() == 250);
    // Update view model properties
    // Update enum by name
    stateProperty->as<rive::ViewModelInstanceEnum>()->value("state-blue");
    // Update trigger
    triggerProperty->as<rive::ViewModelInstanceTrigger>()->propertyValue(1);

    // Advance state machine
    machine->advanceAndApply(0.0f);
    // Validate values have updated
    REQUIRE(rectMappadFill->paint()->as<rive::SolidColor>()->colorValue() ==
            rive::colorARGB(255, 0, 0, 255));
    REQUIRE(shapeMapped->x() == 350);
    REQUIRE(shapeMapped->y() == 250);
    // Update trigger
    triggerProperty->as<rive::ViewModelInstanceTrigger>()->propertyValue(1);

    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(shapeMapped->x() == 350);
    REQUIRE(shapeMapped->y() == 350);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 3 awaits typed Rust execution"]
fn wave_b_data_binding_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("advanceAndApply can skip view model reset", "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test.riv");

    auto artboard = file->artboard("artboard-2")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto machine = artboard->defaultStateMachine();
    REQUIRE(machine != nullptr);
    machine->bindViewModelInstance(viewModelInstance);

    auto triggerProperty = viewModelInstance->propertyValue("trigger-prop");
    REQUIRE(triggerProperty != nullptr);
    REQUIRE(triggerProperty->is<rive::ViewModelInstanceTrigger>());
    auto trigger = triggerProperty->as<rive::ViewModelInstanceTrigger>();

    // Settle initial state.
    machine->advanceAndApply(0.0f);

    // advanceViewModels=false: the bound view model is not consumed, so a
    // trigger set before the advance is retained (the host frame will consume
    // it, not this advance).
    trigger->propertyValue(1);
    machine->advanceAndApply(0.0f, false);
    CHECK(trigger->propertyValue() == 1);

    // The default (advanceViewModels=true) path consumes the trigger, resetting
    // it to 0 via ViewModelInstance::advanced().
    machine->advanceAndApply(0.0f, true);
    CHECK(trigger->propertyValue() == 0);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 4 awaits typed Rust execution"]
fn wave_b_data_binding_test_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("calculate and to string converters with numbers", "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test.riv");

    auto artboard = file->artboard("artboard-3")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto machine = artboard->defaultStateMachine();
    machine->bindViewModelInstance(viewModelInstance);
    REQUIRE(machine != nullptr);
    // Bound property with Calculate Converter (multiply by 2)
    REQUIRE(artboard->find<rive::CustomPropertyNumber>("num_prop") != nullptr);
    auto customPropertyNumber =
        artboard->find<rive::CustomPropertyNumber>("num_prop");
    REQUIRE(customPropertyNumber->propertyValue() == 0.0f);
    // Bound property with Group Converter with:
    // - Calculate (divide by 3)
    // - Convert to string (round decimals and remove trailing zeros)
    REQUIRE(artboard->find<rive::TextValueRun>("text_run_bound") != nullptr);
    auto textRunBound = artboard->find<rive::TextValueRun>("text_run_bound");
    // View model properties
    // Number with initial value set to 17
    auto numProperty = viewModelInstance->propertyValue("num1");
    REQUIRE(numProperty != nullptr);
    REQUIRE(numProperty->is<rive::ViewModelInstanceNumber>());

    REQUIRE(textRunBound->text() == "text");
    // Advance state machine
    machine->advanceAndApply(0.0f);
    // Test Calculate Converter (multiply by 2)
    REQUIRE(customPropertyNumber->propertyValue() == 34.0f);
    REQUIRE(textRunBound->text() == "6");

    // Update value to -10.0f
    numProperty->as<rive::ViewModelInstanceNumber>()->propertyValue(-10.0f);
    // Advance state machine
    machine->advanceAndApply(0.0f);
    // Test Calculate Converter (multiply by 2)
    REQUIRE(customPropertyNumber->propertyValue() == -20.0f);
    REQUIRE(textRunBound->text() == "-3");
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 5 awaits typed Rust execution"]
fn wave_b_data_binding_test_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("trim string converter", "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test.riv");

    auto artboard = file->artboard("artboard-3")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto machine = artboard->defaultStateMachine();
    machine->bindViewModelInstance(viewModelInstance);
    REQUIRE(machine != nullptr);

    REQUIRE(artboard->find<rive::TextValueRun>("second_text_run_trim_both") !=
            nullptr);
    auto trimmedBothTextRunBound =
        artboard->find<rive::TextValueRun>("second_text_run_trim_both");

    REQUIRE(artboard->find<rive::TextValueRun>("second_text_run_trim_start") !=
            nullptr);
    auto trimmedStartTextRunBound =
        artboard->find<rive::TextValueRun>("second_text_run_trim_start");

    REQUIRE(artboard->find<rive::TextValueRun>("second_text_run_trim_end") !=
            nullptr);
    auto trimmedEndTextRunBound =
        artboard->find<rive::TextValueRun>("second_text_run_trim_end");

    REQUIRE(artboard->find<rive::TextValueRun>("second_text_run_no_trim") !=
            nullptr);
    auto notTrimmedTextRunBound =
        artboard->find<rive::TextValueRun>("second_text_run_no_trim");
    // View model properties
    // String with initial value "     abc    "
    auto stringProperty = viewModelInstance->propertyValue("text");
    REQUIRE(stringProperty != nullptr);
    REQUIRE(stringProperty->is<rive::ViewModelInstanceString>());

    REQUIRE(notTrimmedTextRunBound->text() == "text");
    REQUIRE(trimmedBothTextRunBound->text() == "text");
    REQUIRE(trimmedStartTextRunBound->text() == "text");
    REQUIRE(trimmedEndTextRunBound->text() == "text");
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(trimmedBothTextRunBound->text() == "abc");
    REQUIRE(notTrimmedTextRunBound->text() == "     abc    ");
    REQUIRE(trimmedStartTextRunBound->text() == "abc    ");
    REQUIRE(trimmedEndTextRunBound->text() == "     abc");

    // Update value to "a b c "
    stringProperty->as<rive::ViewModelInstanceString>()->propertyValue(
        "a b c ");
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(notTrimmedTextRunBound->text() == "a b c ");
    REQUIRE(trimmedBothTextRunBound->text() == "a b c");
    REQUIRE(trimmedStartTextRunBound->text() == "a b c ");
    REQUIRE(trimmedEndTextRunBound->text() == "a b c");
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 6 awaits typed Rust execution"]
fn wave_b_data_binding_test_006_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("To string converter with color formatters", "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test.riv");

    auto artboard = file->artboard("artboard-4")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto machine = artboard->defaultStateMachine();
    machine->bindViewModelInstance(viewModelInstance);
    REQUIRE(machine != nullptr);

    REQUIRE(artboard->find<rive::TextValueRun>("RGBA_formatted_color_run") !=
            nullptr);
    auto RGBATextRunBound =
        artboard->find<rive::TextValueRun>("RGBA_formatted_color_run");

    REQUIRE(artboard->find<rive::TextValueRun>("rgba_formatted_color_run") !=
            nullptr);
    auto rgbaTextRunBound =
        artboard->find<rive::TextValueRun>("rgba_formatted_color_run");

    REQUIRE(artboard->find<rive::TextValueRun>("hls_formatted_color_run") !=
            nullptr);
    auto hlsTextRunBound =
        artboard->find<rive::TextValueRun>("hls_formatted_color_run");

    REQUIRE(artboard->find<rive::TextValueRun>("escaped_characters_run") !=
            nullptr);
    auto escapedTextRunBound =
        artboard->find<rive::TextValueRun>("escaped_characters_run");

    // View model properties
    // color with initial value "red 30, green 90, blue 200, alpha 255"
    auto colorProperty = viewModelInstance->propertyValue("col");
    REQUIRE(colorProperty != nullptr);
    REQUIRE(colorProperty->is<rive::ViewModelInstanceColor>());

    REQUIRE(RGBATextRunBound->text() == "text");
    REQUIRE(rgbaTextRunBound->text() == "text");
    REQUIRE(hlsTextRunBound->text() == "text");
    REQUIRE(escapedTextRunBound->text() == "text");
    // // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(RGBATextRunBound->text() ==
            "color: {red: 1E, green: 5A, blue: C8, alpha: FF}");
    REQUIRE(rgbaTextRunBound->text() ==
            "color: {red: 30, green: 90, blue: 200, alpha: 255}");
    REQUIRE(hlsTextRunBound->text() ==
            "color: {hue: 219, luminance: 45, saturation: 74}");
    REQUIRE(escapedTextRunBound->text() == "%r %g %b %a \\a");

    // // Update value to "red 200, green 100, blue 50, alpha 100"
    colorProperty->as<rive::ViewModelInstanceColor>()->propertyValue(
        rive::colorARGB(100, 200, 100, 50));
    // // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(RGBATextRunBound->text() ==
            "color: {red: C8, green: 64, blue: 32, alpha: 64}");
    REQUIRE(rgbaTextRunBound->text() ==
            "color: {red: 200, green: 100, blue: 50, alpha: 100}");
    REQUIRE(hlsTextRunBound->text() ==
            "color: {hue: 20, luminance: 49, saturation: 60}");
    REQUIRE(escapedTextRunBound->text() == "%r %g %b %a \\a");

    // // Update value to "red 0, green 10, blue 16, alpha 100"
    colorProperty->as<rive::ViewModelInstanceColor>()->propertyValue(
        rive::colorARGB(100, 0, 10, 15));
    // // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(RGBATextRunBound->text() ==
            "color: {red: 00, green: 0A, blue: 0F, alpha: 64}");
    REQUIRE(rgbaTextRunBound->text() ==
            "color: {red: 0, green: 10, blue: 15, alpha: 100}");
    REQUIRE(hlsTextRunBound->text() ==
            "color: {hue: 200, luminance: 3, saturation: 100}");
    REQUIRE(escapedTextRunBound->text() == "%r %g %b %a \\a");
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 7 awaits typed Rust execution"]
fn wave_b_data_binding_test_007_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Range Mapper", "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test_2.riv");

    auto artboard = file->artboard("artboard-2")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto machine = artboard->defaultStateMachine();
    machine->bindViewModelInstance(viewModelInstance);
    REQUIRE(machine != nullptr);
    // Advance state machine
    machine->advanceAndApply(0.0f);

    // Range mapper [2 - 3]
    REQUIRE(artboard->find<rive::CustomPropertyNumber>("mapped-range-1") !=
            nullptr);
    auto customPropertyNumber1 =
        artboard->find<rive::CustomPropertyNumber>("mapped-range-1");
    REQUIRE(customPropertyNumber1->propertyValue() == 6.0f);

    // Range mapper [2 - 3] - lower clamp - upper clamp
    REQUIRE(artboard->find<rive::CustomPropertyNumber>("mapped-range-2") !=
            nullptr);
    auto customPropertyNumber2 =
        artboard->find<rive::CustomPropertyNumber>("mapped-range-2");
    REQUIRE(customPropertyNumber2->propertyValue() == 3.0f);

    // Range mapper [2 - 3] - modulo
    REQUIRE(artboard->find<rive::CustomPropertyNumber>("mapped-range-3") !=
            nullptr);
    auto customPropertyNumber3 =
        artboard->find<rive::CustomPropertyNumber>("mapped-range-3");
    REQUIRE(customPropertyNumber3->propertyValue() == 2.0f);

    // Range mapper [2 - 3] - lower clamp - upper clamp - reversed
    REQUIRE(artboard->find<rive::CustomPropertyNumber>("mapped-range-4") !=
            nullptr);
    auto customPropertyNumber4 =
        artboard->find<rive::CustomPropertyNumber>("mapped-range-4");
    REQUIRE(customPropertyNumber4->propertyValue() == 2.0f);

    // Range mapper [2 - 2]
    REQUIRE(artboard->find<rive::CustomPropertyNumber>("mapped-range-5") !=
            nullptr);
    auto customPropertyNumber5 =
        artboard->find<rive::CustomPropertyNumber>("mapped-range-5");
    REQUIRE(customPropertyNumber5->propertyValue() == 2.0f);

    // View model properties
    // Number starts at 4.0f
    auto numProperty = viewModelInstance->propertyValue("map-range-num");
    REQUIRE(numProperty != nullptr);
    REQUIRE(numProperty->is<rive::ViewModelInstanceNumber>());
    REQUIRE(numProperty->as<rive::ViewModelInstanceNumber>()->propertyValue() ==
            4.0f);

    // Change value to -1.0f
    numProperty->as<rive::ViewModelInstanceNumber>()->propertyValue(-1.0f);
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(customPropertyNumber1->propertyValue() == 1.0f);
    REQUIRE(customPropertyNumber2->propertyValue() == 2.0f);
    REQUIRE(customPropertyNumber3->propertyValue() == 2.0f);
    REQUIRE(customPropertyNumber4->propertyValue() == 3.0f);
    REQUIRE(customPropertyNumber5->propertyValue() == 2.0f);

    // Change value to 0.0f
    numProperty->as<rive::ViewModelInstanceNumber>()->propertyValue(0.0f);
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(customPropertyNumber1->propertyValue() == 2.0f);
    REQUIRE(customPropertyNumber2->propertyValue() == 2.0f);
    REQUIRE(customPropertyNumber3->propertyValue() == 2.0f);
    REQUIRE(customPropertyNumber4->propertyValue() == 3.0f);
    REQUIRE(customPropertyNumber5->propertyValue() == 2.0f);

    // Change value to 0.25f
    numProperty->as<rive::ViewModelInstanceNumber>()->propertyValue(0.25f);
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(customPropertyNumber1->propertyValue() == Approx(2.12916f));
    REQUIRE(customPropertyNumber2->propertyValue() == Approx(2.12916f));
    REQUIRE(customPropertyNumber3->propertyValue() == Approx(2.12916f));
    REQUIRE(customPropertyNumber4->propertyValue() == Approx(2.87084f));
    REQUIRE(customPropertyNumber5->propertyValue() == 2.0f);

    // Change value to 2.0f
    numProperty->as<rive::ViewModelInstanceNumber>()->propertyValue(2.0f);
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(customPropertyNumber1->propertyValue() == 4.0f);
    REQUIRE(customPropertyNumber2->propertyValue() == 3.0f);
    REQUIRE(customPropertyNumber3->propertyValue() == 2.0f);
    REQUIRE(customPropertyNumber4->propertyValue() == 2.0f);
    REQUIRE(customPropertyNumber5->propertyValue() == 2.0f);

    // Change value to 2.25f
    numProperty->as<rive::ViewModelInstanceNumber>()->propertyValue(2.25f);
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(customPropertyNumber1->propertyValue() == 4.25f);
    REQUIRE(customPropertyNumber2->propertyValue() == 3.0f);
    REQUIRE(customPropertyNumber3->propertyValue() == Approx(2.12916f));
    REQUIRE(customPropertyNumber4->propertyValue() == 2.0f);
    REQUIRE(customPropertyNumber5->propertyValue() == 2.0f);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 8 awaits typed Rust execution"]
fn wave_b_data_binding_test_008_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Pad String", "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test_2.riv");

    auto artboard = file->artboard("artboard-3")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto machine = artboard->defaultStateMachine();
    machine->bindViewModelInstance(viewModelInstance);
    REQUIRE(machine != nullptr);
    // Advance state machine
    machine->advanceAndApply(0.0f);

    // Pad string "abc" - length: 11 - Start
    REQUIRE(artboard->find<rive::CustomPropertyString>("pad-string-1") !=
            nullptr);
    auto customPropertyString1 =
        artboard->find<rive::CustomPropertyString>("pad-string-1");
    REQUIRE(customPropertyString1->propertyValue() == "abcabcatext");

    // Pad string "abc" - length: 12 - End
    REQUIRE(artboard->find<rive::CustomPropertyString>("pad-string-2") !=
            nullptr);
    auto customPropertyString2 =
        artboard->find<rive::CustomPropertyString>("pad-string-2");
    REQUIRE(customPropertyString2->propertyValue() == "textabcabcab");

    // Pad string "abc" - length: 12 - End - Worng type of input
    REQUIRE(artboard->find<rive::CustomPropertyString>("pad-string-3") !=
            nullptr);
    auto customPropertyString3 =
        artboard->find<rive::CustomPropertyString>("pad-string-3");
    REQUIRE(customPropertyString3->propertyValue() == "");

    // View model properties
    // String starts with "text"
    auto stringProperty = viewModelInstance->propertyValue("pad-string");
    REQUIRE(stringProperty != nullptr);
    REQUIRE(stringProperty->is<rive::ViewModelInstanceString>());
    REQUIRE(
        stringProperty->as<rive::ViewModelInstanceString>()->propertyValue() ==
        "text");

    // Change value to "text-text-text", longer than length
    stringProperty->as<rive::ViewModelInstanceString>()->propertyValue(
        "text-text-text");
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(customPropertyString1->propertyValue() == "text-text-text");
    REQUIRE(customPropertyString2->propertyValue() == "text-text-text");
    REQUIRE(customPropertyString3->propertyValue() == "");

    // Change value to "", empty string
    stringProperty->as<rive::ViewModelInstanceString>()->propertyValue("");
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(customPropertyString1->propertyValue() == "abcabcabcab");
    REQUIRE(customPropertyString2->propertyValue() == "abcabcabcabc");
    REQUIRE(customPropertyString3->propertyValue() == "");
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 9 awaits typed Rust execution"]
fn wave_b_data_binding_test_009_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Boolean Toggle", "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test_2.riv");

    auto artboard = file->artboard("artboard-3")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto machine = artboard->defaultStateMachine();
    machine->bindViewModelInstance(viewModelInstance);
    REQUIRE(machine != nullptr);
    // Advance state machine
    machine->advanceAndApply(0.0f);

    // Boolean property
    REQUIRE(artboard->find<rive::CustomPropertyBoolean>("negate-bool-1") !=
            nullptr);
    auto customPropertyBoolean1 =
        artboard->find<rive::CustomPropertyBoolean>("negate-bool-1");
    REQUIRE(customPropertyBoolean1->propertyValue() == true);

    // View model properties
    // bool property starts as false
    auto boolProperty = viewModelInstance->propertyValue("bool-prop");
    REQUIRE(boolProperty != nullptr);
    REQUIRE(boolProperty->is<rive::ViewModelInstanceBoolean>());
    REQUIRE(
        boolProperty->as<rive::ViewModelInstanceBoolean>()->propertyValue() ==
        false);

    // Change value to true
    boolProperty->as<rive::ViewModelInstanceBoolean>()->propertyValue(true);
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(customPropertyBoolean1->propertyValue() == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 10 awaits typed Rust execution"]
fn wave_b_data_binding_test_010_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Instance is shared when the same one is attached to two properties",
          "[data binding]")
{
    auto file = ReadRiveFile("assets/shared_viewmodel_instance.riv");

    auto artboard = file->artboard("main")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto machine = artboard->defaultStateMachine();
    machine->bindViewModelInstance(viewModelInstance);
    REQUIRE(machine != nullptr);
    // Advance state machine
    machine->advanceAndApply(0.0f);

    // View model properties
    auto childInstanceViewModel1 = viewModelInstance->propertyValue("child1");
    REQUIRE(childInstanceViewModel1 != nullptr);
    REQUIRE(childInstanceViewModel1->is<rive::ViewModelInstanceViewModel>());
    auto referencedViewModel =
        childInstanceViewModel1->as<rive::ViewModelInstanceViewModel>()
            ->referenceViewModelInstance();
    REQUIRE(referencedViewModel != nullptr);
    auto labelProperty = referencedViewModel->propertyValue("label");
    REQUIRE(labelProperty != nullptr);
    REQUIRE(labelProperty->is<rive::ViewModelInstanceString>());

    // Elements bound to different view model properties that share same view
    // model instance
    REQUIRE(artboard->find<rive::NestedArtboard>("child1") != nullptr);
    auto nestedArtboardChild1 = artboard->find<rive::NestedArtboard>("child1");

    auto nestedArtboardArtboardChild1 =
        nestedArtboardChild1->artboardInstance();
    REQUIRE(nestedArtboardArtboardChild1 != nullptr);
    auto textRunChild1 =
        nestedArtboardArtboardChild1->find<rive::TextValueRun>("text_run");
    REQUIRE(textRunChild1 != nullptr);
    REQUIRE(textRunChild1->text() == "label-vmi-1");

    REQUIRE(artboard->find<rive::NestedArtboard>("child2") != nullptr);
    auto nestedArtboardChild2 = artboard->find<rive::NestedArtboard>("child2");

    auto nestedArtboardArtboardChild2 =
        nestedArtboardChild2->artboardInstance();
    REQUIRE(nestedArtboardArtboardChild2 != nullptr);
    auto textRunChild2 =
        nestedArtboardArtboardChild2->find<rive::TextValueRun>("text_run");
    REQUIRE(textRunChild2 != nullptr);
    REQUIRE(textRunChild2->text() == "label-vmi-1");

    // Changing the value on a single instance should affect both text although
    // they are linked to different view model properties
    labelProperty->as<rive::ViewModelInstanceString>()->propertyValue(
        "label-update");
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(textRunChild1->text() == "label-update");
    REQUIRE(textRunChild2->text() == "label-update");
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 11 awaits typed Rust execution"]
fn wave_b_data_binding_test_011_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Instance is not shared when the different ones are attached to two "
          "properties",
          "[data binding]")
{
    auto file = ReadRiveFile("assets/shared_viewmodel_instance.riv");

    auto artboard = file->artboard("main_2")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto machine = artboard->defaultStateMachine();
    machine->bindViewModelInstance(viewModelInstance);
    REQUIRE(machine != nullptr);
    // Advance state machine
    machine->advanceAndApply(0.0f);

    // View model properties
    auto childInstanceViewModel1 =
        viewModelInstance->propertyValue("vm_2_child1");
    REQUIRE(childInstanceViewModel1 != nullptr);
    REQUIRE(childInstanceViewModel1->is<rive::ViewModelInstanceViewModel>());
    auto referencedViewModel =
        childInstanceViewModel1->as<rive::ViewModelInstanceViewModel>()
            ->referenceViewModelInstance();
    REQUIRE(referencedViewModel != nullptr);
    auto labelProperty = referencedViewModel->propertyValue("label");
    REQUIRE(labelProperty != nullptr);
    REQUIRE(labelProperty->is<rive::ViewModelInstanceString>());

    // Elements bound to different view model properties that do not share same
    // view model instance
    REQUIRE(artboard->find<rive::NestedArtboard>("child1") != nullptr);
    auto nestedArtboardChild1 = artboard->find<rive::NestedArtboard>("child1");

    auto nestedArtboardArtboardChild1 =
        nestedArtboardChild1->artboardInstance();
    REQUIRE(nestedArtboardArtboardChild1 != nullptr);
    auto textRunChild1 =
        nestedArtboardArtboardChild1->find<rive::TextValueRun>("text_run");
    REQUIRE(textRunChild1 != nullptr);
    REQUIRE(textRunChild1->text() == "label-vmi-1");

    REQUIRE(artboard->find<rive::NestedArtboard>("child2") != nullptr);
    auto nestedArtboardChild2 = artboard->find<rive::NestedArtboard>("child2");

    auto nestedArtboardArtboardChild2 =
        nestedArtboardChild2->artboardInstance();
    REQUIRE(nestedArtboardArtboardChild2 != nullptr);
    auto textRunChild2 =
        nestedArtboardArtboardChild2->find<rive::TextValueRun>("text_run");
    REQUIRE(textRunChild2 != nullptr);
    REQUIRE(textRunChild2->text() == "label-vmi-2");

    // Changing the value on a single instance should not affect the other
    // instance because they are not pointing to the same instance
    labelProperty->as<rive::ViewModelInstanceString>()->propertyValue(
        "label-update");
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(textRunChild1->text() == "label-update");
    REQUIRE(textRunChild2->text() == "label-vmi-2");
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 12 awaits typed Rust execution"]
fn wave_b_data_binding_test_012_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Instances are not shared when a new view model instance is created",
          "[data binding]")
{
    auto file = ReadRiveFile("assets/shared_viewmodel_instance.riv");

    auto artboard = file->artboard("main_2")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance = file->createViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto machine = artboard->defaultStateMachine();
    machine->bindViewModelInstance(viewModelInstance);
    REQUIRE(machine != nullptr);
    // Advance state machine
    machine->advanceAndApply(0.0f);

    // View model properties
    auto childInstanceViewModel1 =
        viewModelInstance->propertyValue("vm_2_child1");
    REQUIRE(childInstanceViewModel1 != nullptr);
    REQUIRE(childInstanceViewModel1->is<rive::ViewModelInstanceViewModel>());
    auto referencedViewModel =
        childInstanceViewModel1->as<rive::ViewModelInstanceViewModel>()
            ->referenceViewModelInstance();
    REQUIRE(referencedViewModel != nullptr);
    auto labelProperty = referencedViewModel->propertyValue("label");
    REQUIRE(labelProperty != nullptr);
    REQUIRE(labelProperty->is<rive::ViewModelInstanceString>());

    // Elements bound to different view model properties that do not share same
    // view model instance
    REQUIRE(artboard->find<rive::NestedArtboard>("child1") != nullptr);
    auto nestedArtboardChild1 = artboard->find<rive::NestedArtboard>("child1");

    auto nestedArtboardArtboardChild1 =
        nestedArtboardChild1->artboardInstance();
    REQUIRE(nestedArtboardArtboardChild1 != nullptr);
    auto textRunChild1 =
        nestedArtboardArtboardChild1->find<rive::TextValueRun>("text_run");
    REQUIRE(textRunChild1 != nullptr);
    REQUIRE(textRunChild1->text() == "");

    REQUIRE(artboard->find<rive::NestedArtboard>("child2") != nullptr);
    auto nestedArtboardChild2 = artboard->find<rive::NestedArtboard>("child2");

    auto nestedArtboardArtboardChild2 =
        nestedArtboardChild2->artboardInstance();
    REQUIRE(nestedArtboardArtboardChild2 != nullptr);
    auto textRunChild2 =
        nestedArtboardArtboardChild2->find<rive::TextValueRun>("text_run");
    REQUIRE(textRunChild2 != nullptr);
    REQUIRE(textRunChild2->text() == "");

    // Changing the value on a single instance should not affect the other
    // instance because they are not pointing to the same instance
    labelProperty->as<rive::ViewModelInstanceString>()->propertyValue(
        "label-update");
    // Advance state machine
    machine->advanceAndApply(0.0f);
    REQUIRE(textRunChild1->text() == "label-update");
    REQUIRE(textRunChild2->text() == "");
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 13 awaits typed Rust execution"]
fn wave_b_data_binding_test_013_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Triggers updated by events correctly update state", "[data binding]")
{
    auto file = ReadRiveFile("assets/data_binding_test_triggers.riv");

    auto artboard = file->artboard("root")->instance();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance = file->createViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto machine = artboard->defaultStateMachine();
    machine->bindViewModelInstance(viewModelInstance);
    REQUIRE(machine != nullptr);
    // Advance state machine
    machine->advanceAndApply(0.0f);

    REQUIRE(artboard->find<rive::Shape>("main_rect") != nullptr);
    auto rect = artboard->find<rive::Shape>("main_rect");

    REQUIRE(rect->children()[1]->is<rive::Fill>());
    rive::Fill* rectMappadFill = rect->children()[1]->as<rive::Fill>();
    REQUIRE(rectMappadFill->paint()->is<rive::SolidColor>());
    REQUIRE(rectMappadFill->paint()->as<rive::SolidColor>()->colorValue() ==
            rive::colorARGB(255, 255, 0, 0));

    // Advance state machine so the child reports the event
    machine->advanceAndApply(0.7f);
    // Advance state machine so the parent consumes the event
    machine->advanceAndApply(0.1f);
    REQUIRE(rectMappadFill->paint()->as<rive::SolidColor>()->colorValue() ==
            rive::colorARGB(255, 0, 255, 0));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 14 awaits typed Rust execution"]
fn wave_b_data_binding_test_014_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Transition self conditions", "[data binding]")
{

    rive::SerializingFactory silver;
    auto file =
        ReadRiveFile("assets/transition_self_comparator_test.riv", &silver);

    auto artboard = file->artboardDefault();

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());
    auto numProp =
        vmi->propertyValue("num")->as<rive::ViewModelInstanceNumber>();
    auto triggerProp =
        vmi->propertyValue("tri")->as<rive::ViewModelInstanceTrigger>();
    auto colorProp =
        vmi->propertyValue("col")->as<rive::ViewModelInstanceColor>();
    auto bolProp =
        vmi->propertyValue("bol")->as<rive::ViewModelInstanceBoolean>();
    auto stringProp =
        vmi->propertyValue("str")->as<rive::ViewModelInstanceString>();
    auto lisProp = vmi->propertyValue("lis")->as<rive::ViewModelInstanceList>();

    // Number value changes once triggering a state transition
    silver.addFrame();
    numProp->propertyValue(20);
    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());

    // Setting number property to same value doesn't trigger state transition
    silver.addFrame();
    numProp->propertyValue(20);
    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());

    // Updating number property twice triggers single state transition
    silver.addFrame();
    numProp->propertyValue(10);
    numProp->propertyValue(20);
    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());

    // Updating two properties triggers transition in different layers
    silver.addFrame();
    numProp->propertyValue(10);
    triggerProp->trigger();
    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());

    // Updating color property twice triggers state transition
    silver.addFrame();
    colorProp->propertyValue(rive::colorARGB(100, 0, 10, 15));
    colorProp->propertyValue(rive::colorARGB(101, 0, 10, 15));
    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());

    // Updating color property once more triggers state transition
    silver.addFrame();
    colorProp->propertyValue(rive::colorARGB(102, 0, 10, 15));
    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());

    // Updating boolean property twice triggers state transition
    silver.addFrame();
    bolProp->propertyValue(true);
    bolProp->propertyValue(false);
    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());

    // Updating boolean property once more triggers state transition
    silver.addFrame();
    bolProp->propertyValue(true);
    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());

    // Updating string property twice triggers state transition
    silver.addFrame();
    stringProp->propertyValue("a");
    stringProp->propertyValue("b");
    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());

    // Updating string property once more triggers state transition
    silver.addFrame();
    stringProp->propertyValue("c");
    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());

    // Updating list property by adding two items to the list triggers a single
    // state transition
    {
        silver.addFrame();
        auto itemList = rive::make_rcp<rive::ViewModelInstanceListItem>();
        lisProp->addItem(itemList);
        auto itemList2 = rive::make_rcp<rive::ViewModelInstanceListItem>();
        lisProp->addItem(itemList2);
        stateMachine->advanceAndApply(0.0f);
        artboard->draw(renderer.get());
    }

    // Updating list property by adding one item to the list triggers a single
    // state transition
    {
        silver.addFrame();
        auto itemList = rive::make_rcp<rive::ViewModelInstanceListItem>();
        lisProp->addItem(itemList);
        stateMachine->advanceAndApply(0.0f);
        artboard->draw(renderer.get());
    }

    // Updating list property by adding one item at a specific position triggers
    // a state transition
    {
        silver.addFrame();
        auto itemList = rive::make_rcp<rive::ViewModelInstanceListItem>();
        lisProp->addItemAt(itemList, 0);
        stateMachine->advanceAndApply(0.0f);
        artboard->draw(renderer.get());
    }

    // Updating list property by adding one item at a at an invalid position
    // does not trigger a state transition
    {
        silver.addFrame();
        auto itemList = rive::make_rcp<rive::ViewModelInstanceListItem>();
        lisProp->addItemAt(itemList, 10);
        stateMachine->advanceAndApply(0.0f);
        artboard->draw(renderer.get());
    }

    // Updating list property swapping items triggers a state transition
    {
        silver.addFrame();
        lisProp->swap(0, 1);
        stateMachine->advanceAndApply(0.0f);
        artboard->draw(renderer.get());
    }

    // Removing item from list by index triggers a state transition
    {
        silver.addFrame();
        lisProp->removeItem(0);
        stateMachine->advanceAndApply(0.0f);
        artboard->draw(renderer.get());
    }

    // Removing item from list by index outside of range doesn't trigger a state
    // transition
    {
        silver.addFrame();
        lisProp->removeItem(10);
        stateMachine->advanceAndApply(0.0f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("transition_self_comparator_test"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 15 awaits typed Rust execution"]
fn wave_b_data_binding_test_015_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Computed root transform nested artboard", "[data binding]")
{

    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/computed_root_transform.riv", &silver);

    auto artboard = file->artboardNamed("nested-artboard-main");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = (int)(1.0f / 0.016f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("computed_root_transform-nested_artboard"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 16 awaits typed Rust execution"]
fn wave_b_data_binding_test_016_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Computed root transform list", "[data binding]")
{

    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/computed_root_transform.riv", &silver);

    auto artboard = file->artboardNamed("list-main");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = (int)(1.0f / 0.016f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("computed_root_transform-list"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 17 awaits typed Rust execution"]
fn wave_b_data_binding_test_017_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Trigger based listeners", "[data binding]")
{

    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/trigger_based_listeners.riv", &silver);

    auto artboard = file->artboardNamed("main");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());
    stateMachine->pointerDown(rive::Vec2D(25.0f, 25.0f));
    // Advance and apply twice to take the transition and apply the next state.
    stateMachine->advanceAndApply(0.1f);
    stateMachine->advanceAndApply(1.0f);
    stateMachine->pointerUp(rive::Vec2D(25.0f, 25.0f));
    // Advance and apply twice to take the transition and apply the next state.
    stateMachine->advanceAndApply(0.1f);
    stateMachine->advanceAndApply(1.0f);

    silver.addFrame();
    artboard->draw(renderer.get());

    CHECK(silver.matches("trigger_based_listeners"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 18 awaits typed Rust execution"]
fn wave_b_data_binding_test_018_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Custom Property Trigger Binding", "[data binding]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/custom_property_trigger.riv", &silver);

    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    silver.frameSize(artboard->width(), artboard->height());

    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    auto machine = artboard->defaultStateMachine();
    machine->bindViewModelInstance(viewModelInstance);
    REQUIRE(machine != nullptr);
    // Advance state machine
    machine->advanceAndApply(0.0f);

    auto circle = artboard->find<rive::Shape>("MainCircle");
    REQUIRE(circle != nullptr);
    REQUIRE(circle->scaleX() == 1.0f);
    REQUIRE(circle->scaleY() == 1.0f);

    auto trig = artboard->find<rive::CustomPropertyTrigger>("Trig");
    REQUIRE(trig != nullptr);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = (int)(1.0f / 0.16f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        machine->advanceAndApply(0.16f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("custom_property_trigger_bind"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 19 awaits typed Rust execution"]
fn wave_b_data_binding_test_019_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Data binding solos", "[data binding]")
{

    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/data_bind_solo.riv", &silver);

    auto artboard = file->artboardNamed("values-to-solos");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = (int)(1.0f / 0.016f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("data_bind_solo-values-to-solos"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 20 awaits typed Rust execution"]
fn wave_b_data_binding_test_020_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Data binding solos - target to source", "[data binding]")
{

    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/data_bind_solo.riv", &silver);

    auto artboard = file->artboardNamed("solos-to-values");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = (int)(1.0f / 0.016f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("data_bind_solo-solos-to-values"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 21 awaits typed Rust execution"]
fn wave_b_data_binding_test_021_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("State machine fire triggers", "[data binding]")
{

    rive::SerializingFactory silver;
    auto file =
        ReadRiveFile("assets/state_transition_fire_trigger.riv", &silver);

    auto artboard = file->artboardNamed("main");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());
    // Advance and apply twice to take the transition and apply the next state.
    stateMachine->advanceAndApply(0.1f);
    stateMachine->advanceAndApply(1.0f);

    silver.addFrame();
    stateMachine->advanceAndApply(0.1f);
    stateMachine->advanceAndApply(1.0f);
    artboard->draw(renderer.get());

    silver.addFrame();
    // Advance and apply twice to take the transition and apply the next state.
    stateMachine->advanceAndApply(0.1f);
    stateMachine->advanceAndApply(1.0f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("state_transition_fire_trigger"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 22 awaits typed Rust execution"]
fn wave_b_data_binding_test_022_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Custom enum properties", "[data binding]")
{

    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/custom_property_enum.riv", &silver);

    auto artboard = file->artboardNamed("main");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = (int)(3.0f / 0.048f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.048f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("custom_property_enum"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 23 awaits typed Rust execution"]
fn wave_b_data_binding_test_023_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("View model runtime properties", "[data binding]")
{

    auto file = ReadRiveFile("assets/viewmodel_runtime_file.riv");

    auto vm = file->viewModelByName("vm");
    REQUIRE(vm != nullptr);
    auto instance = vm->createDefaultInstance();
    REQUIRE(instance != nullptr);
    // Grab ViewModel name from instance
    auto viewModelName = instance->viewModelName();
    REQUIRE(viewModelName == "vm");
    // Number
    auto num = instance->propertyNumber("num");
    REQUIRE(num != nullptr);
    REQUIRE(num->dataType() == rive::DataType::number);
    // string
    auto str = instance->propertyString("str");
    REQUIRE(str != nullptr);
    REQUIRE(str->dataType() == rive::DataType::string);
    // Requesting a property with the same name from cache doesn't return the
    // wrong object
    auto strWrong = instance->propertyNumber("str");
    REQUIRE(strWrong == nullptr);
    // Boolean
    auto boo = instance->propertyBoolean("boo");
    REQUIRE(boo != nullptr);
    REQUIRE(boo->dataType() == rive::DataType::boolean);
    // Color
    auto col = instance->propertyColor("col");
    REQUIRE(col != nullptr);
    REQUIRE(col->dataType() == rive::DataType::color);
    // Trigger
    auto tri = instance->propertyTrigger("tri");
    REQUIRE(tri != nullptr);
    REQUIRE(tri->dataType() == rive::DataType::trigger);
    // Enum
    auto enu = instance->propertyEnum("enu");
    REQUIRE(enu != nullptr);
    REQUIRE(enu->dataType() == rive::DataType::enumType);
    // Image
    auto ima = instance->propertyImage("ima");
    REQUIRE(ima != nullptr);
    REQUIRE(ima->dataType() == rive::DataType::assetImage);
    // Artboard
    auto art = instance->propertyArtboard("art");
    REQUIRE(art != nullptr);
    REQUIRE(art->dataType() == rive::DataType::artboard);
    // List
    auto lis = instance->propertyList("lis");
    REQUIRE(lis != nullptr);
    REQUIRE(lis->dataType() == rive::DataType::list);
    // number in nested view model: chi > num
    auto numChi = instance->propertyNumber("chi/chi-num");
    REQUIRE(numChi != nullptr);
    REQUIRE(numChi->dataType() == rive::DataType::number);

    // Enum properties expose the backing enum's name while non-enum properties
    // leave enumName empty.
    auto properties = instance->properties();
    auto findProperty = [&properties](const std::string& name) {
        return std::find_if(properties.begin(),
                            properties.end(),
                            [&name](const rive::PropertyData& data) {
                                return data.name == name;
                            });
    };
    auto enuData = findProperty("enu");
    REQUIRE(enuData != properties.end());
    REQUIRE(enuData->type == rive::DataType::enumType);
    REQUIRE(enuData->enumName == "Horizontal Align");

    auto numData = findProperty("num");
    REQUIRE(numData != properties.end());
    REQUIRE(numData->type == rive::DataType::number);
    REQUIRE(numData->enumName.empty());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 24 awaits typed Rust execution"]
fn wave_b_data_binding_test_024_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Trigger fires single change on listener", "[data binding]")
{

    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/trigger_fires_single_change.riv", &silver);

    auto artboard = file->artboardNamed("main");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());
    // Advance and apply twice to take the transition and apply the next state.
    stateMachine->pointerDown(rive::Vec2D(225.0f, 275.0f));
    stateMachine->advanceAndApply(0.1f);
    stateMachine->advanceAndApply(1.0f);
    stateMachine->pointerUp(rive::Vec2D(225.0f, 275.0f));
    stateMachine->advanceAndApply(0.1f);
    stateMachine->advanceAndApply(1.0f);
    silver.addFrame();
    artboard->draw(renderer.get());

    // Advance and apply twice to take the transition and apply the next state.
    stateMachine->pointerDown(rive::Vec2D(225.0f, 275.0f));
    stateMachine->advanceAndApply(0.1f);
    stateMachine->advanceAndApply(1.0f);
    stateMachine->pointerUp(rive::Vec2D(225.0f, 275.0f));
    stateMachine->advanceAndApply(0.1f);
    stateMachine->advanceAndApply(1.0f);
    silver.addFrame();
    artboard->draw(renderer.get());

    CHECK(silver.matches("trigger_fires_single_change"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 25 awaits typed Rust execution"]
fn wave_b_data_binding_test_025_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Convert to number", "[data binding]")
{

    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/data_converter_to_number.riv", &silver);

    auto artboard = file->artboardNamed("main");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = (int)(1.2f / 0.016f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("data_converter_to_number"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 26 awaits typed Rust execution"]
fn wave_b_data_binding_test_026_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("List to path", "[data binding]")
{

    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/list_to_path.riv", &silver);

    auto artboard = file->artboardNamed("main");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);
    auto listProp =
        vmi->propertyValue("lis")->as<rive::ViewModelInstanceList>();

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    silver.addFrame();
    // Create a square with 4 XY vertices

    auto vertex1 = file->createViewModelInstance("vertex-x-y");
    auto vertexInstanceListItem1 =
        rive::make_rcp<rive::ViewModelInstanceListItem>();
    vertexInstanceListItem1->viewModelInstance(vertex1);
    listProp->addItem(vertexInstanceListItem1);

    auto vertex2 = file->createViewModelInstance("vertex-x-y");
    vertex2->propertyValue("x")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(100);
    auto vertexInstanceListItem2 =
        rive::make_rcp<rive::ViewModelInstanceListItem>();
    vertexInstanceListItem2->viewModelInstance(vertex2);
    listProp->addItem(vertexInstanceListItem2);

    auto vertex3 = file->createViewModelInstance("vertex-x-y");
    vertex3->propertyValue("x")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(100);
    vertex3->propertyValue("y")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(100);
    auto vertexInstanceListItem3 =
        rive::make_rcp<rive::ViewModelInstanceListItem>();
    vertexInstanceListItem3->viewModelInstance(vertex3);
    listProp->addItem(vertexInstanceListItem3);

    auto vertex4 = file->createViewModelInstance("vertex-x-y");
    vertex4->propertyValue("y")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(100);
    auto vertexInstanceListItem4 =
        rive::make_rcp<rive::ViewModelInstanceListItem>();
    vertexInstanceListItem4->viewModelInstance(vertex4);
    listProp->addItem(vertexInstanceListItem4);

    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    silver.addFrame();
    // Insert a mirrored vertex at index 2
    auto vertexRD1 = file->createViewModelInstance("vertex-rotation-distance");
    vertexRD1->propertyValue("x")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(200);
    vertexRD1->propertyValue("rotation")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(1.5f);
    vertexRD1->propertyValue("distance")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(20);
    auto vertexInstanceListItemRD1 =
        rive::make_rcp<rive::ViewModelInstanceListItem>();
    vertexInstanceListItemRD1->viewModelInstance(vertexRD1);
    listProp->addItemAt(vertexInstanceListItemRD1, 2);

    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    silver.addFrame();
    // Insert a detached vertex at index 3
    auto vertexD1 = file->createViewModelInstance("vertex-detached");
    vertexD1->propertyValue("x")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(200);
    vertexD1->propertyValue("y")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(100);
    vertexD1->propertyValue("inRotation")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(1);
    vertexD1->propertyValue("outRotation")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(2);
    vertexD1->propertyValue("inDistance")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(10);
    vertexD1->propertyValue("outDistance")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(30);
    auto vertexInstanceListItemD1 =
        rive::make_rcp<rive::ViewModelInstanceListItem>();
    vertexInstanceListItemD1->viewModelInstance(vertexD1);
    listProp->addItemAt(vertexInstanceListItemD1, 3);

    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    silver.addFrame();
    // Insert a cubic in out vertex at index 4
    auto vertexIO1 = file->createViewModelInstance("vertex-in-out");
    vertexIO1->propertyValue("x")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(100);
    vertexIO1->propertyValue("y")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(200);
    vertexIO1->propertyValue("inX")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(40);
    vertexIO1->propertyValue("inY")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(20);
    vertexIO1->propertyValue("outX")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(10);
    vertexIO1->propertyValue("outY")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(30);
    auto vertexInstanceListItemIO1 =
        rive::make_rcp<rive::ViewModelInstanceListItem>();
    vertexInstanceListItemIO1->viewModelInstance(vertexIO1);
    listProp->addItemAt(vertexInstanceListItemIO1, 4);

    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    silver.addFrame();
    // Insert a non valid vertex at index 4
    auto vertexN1 = file->createViewModelInstance("non-vertex");
    auto vertexInstanceListItemN1 =
        rive::make_rcp<rive::ViewModelInstanceListItem>();
    vertexInstanceListItemN1->viewModelInstance(vertexN1);
    listProp->addItemAt(vertexInstanceListItemN1, 5);

    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    silver.addFrame();
    // Insert a vertex with some paired values undefined
    auto vertexI1 = file->createViewModelInstance("vertex-incomplete");

    vertexI1->propertyValue("x")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(100);
    vertexI1->propertyValue("y")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(300);
    vertexI1->propertyValue("inDistance")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(60);
    vertexI1->propertyValue("inRotation")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(-1);
    vertexI1->propertyValue("outX")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(30);
    vertexI1->propertyValue("inX")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(-30);
    auto vertexInstanceListItemI1 =
        rive::make_rcp<rive::ViewModelInstanceListItem>();
    vertexInstanceListItemI1->viewModelInstance(vertexI1);
    listProp->addItemAt(vertexInstanceListItemI1, 4);

    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    silver.addFrame();
    // Update some values to trigger dirt updates
    vertexI1->propertyValue("inX")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(-30);

    vertex1->propertyValue("x")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(50);

    vertexRD1->propertyValue("rotation")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(1.0f);

    vertexD1->propertyValue("inDistance")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(30);

    vertexIO1->propertyValue("outY")
        ->as<rive::ViewModelInstanceNumber>()
        ->propertyValue(40);

    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    for (int i = 0; i < 60; i++)
    {
        silver.addFrame();
        vertexI1->propertyValue("inRotation")
            ->as<rive::ViewModelInstanceNumber>()
            ->propertyValue((float)i * 6);
        vertexRD1->propertyValue("rotation")
            ->as<rive::ViewModelInstanceNumber>()
            ->propertyValue((float)i * 6);
        stateMachine->advanceAndApply(0.01f);
        stateMachine->advanceAndApply(0.0f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("list_to_path"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 27 awaits typed Rust execution"]
fn wave_b_data_binding_test_027_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Format text with commas", "[data binding]")
{

    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/format_number_with_commas.riv", &silver);

    auto artboard = file->artboardNamed("main");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    CHECK(silver.matches("format_number_with_commas"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 28 awaits typed Rust execution"]
fn wave_b_data_binding_test_028_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Interpolate color and number", "[data binding]")
{

    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/time_based_interpolation.riv", &silver);

    auto artboard = file->artboardNamed("main");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    stateMachine->pointerDown(rive::Vec2D(25.0f, 25.0f));
    // Advance and apply twice to take the transition and apply the next state.
    stateMachine->advanceAndApply(0.016f);
    stateMachine->advanceAndApply(0.016f);
    stateMachine->pointerUp(rive::Vec2D(25.0f, 25.0f));
    // Advance and apply twice to take the transition and apply the next state.
    stateMachine->advanceAndApply(0.016f);
    stateMachine->advanceAndApply(0.016f);

    silver.addFrame();
    artboard->draw(renderer.get());

    int frames = (int)(1.0f / 0.032f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.032f);
        artboard->draw(renderer.get());
    }

    stateMachine->pointerDown(rive::Vec2D(425.0f, 25.0f));
    // Advance and apply twice to take the transition and apply the next state.
    stateMachine->advanceAndApply(0.016f);
    stateMachine->advanceAndApply(0.016f);
    stateMachine->pointerUp(rive::Vec2D(425.0f, 25.0f));
    // Advance and apply twice to take the transition and apply the next state.
    stateMachine->advanceAndApply(0.016f);
    stateMachine->advanceAndApply(0.016f);

    silver.addFrame();
    artboard->draw(renderer.get());

    for (int i = 0; i < 10; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.032f);
        artboard->draw(renderer.get());
    }

    // Interrupt interpolation mid way
    stateMachine->pointerDown(rive::Vec2D(25.0f, 25.0f));
    // Advance and apply twice to take the transition and apply the next state.
    stateMachine->advanceAndApply(0.016f);
    stateMachine->advanceAndApply(0.016f);
    stateMachine->pointerUp(rive::Vec2D(25.0f, 25.0f));
    // Advance and apply twice to take the transition and apply the next state.
    stateMachine->advanceAndApply(0.016f);
    stateMachine->advanceAndApply(0.016f);

    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.032f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("time_based_interpolation"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 29 awaits typed Rust execution"]
fn wave_b_data_binding_test_029_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Bidirectional data binding with source to target precedence",
          "[data binding]")
{

    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/bidirectional_precedence.riv", &silver);

    auto artboard = file->artboardNamed("source_first");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    auto xProp = vmi->propertyValue("x")->as<rive::ViewModelInstanceNumber>();
    auto yProp = vmi->propertyValue("y")->as<rive::ViewModelInstanceNumber>();

    // On source first these values will overwrite the target values
    // that are [250,250]
    xProp->propertyValue(100);
    yProp->propertyValue(100);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    CHECK(silver.matches("bidirectional_precedence-source_first"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 30 awaits typed Rust execution"]
fn wave_b_data_binding_test_030_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Bidirectional data binding with target to source precedence",
          "[data binding]")
{

    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/bidirectional_precedence.riv", &silver);

    auto artboard = file->artboardNamed("target_first");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    auto xProp = vmi->propertyValue("x")->as<rive::ViewModelInstanceNumber>();
    auto yProp = vmi->propertyValue("y")->as<rive::ViewModelInstanceNumber>();

    // On target first these values will be overwritten by the target values
    // that are [250,250]
    xProp->propertyValue(100);
    yProp->propertyValue(100);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    CHECK(silver.matches("bidirectional_precedence-target_first"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 31 awaits typed Rust execution"]
fn wave_b_data_binding_test_031_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("TwoWay source change reaches target under target-first precedence",
          "[data binding]")
{
    auto file = ReadRiveFile("assets/bidirectional_precedence.riv");
    auto artboard = file->artboardNamed("target_first");
    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();
    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    auto xProp = vmi->propertyValue("x")->as<rive::ViewModelInstanceNumber>();
    auto yProp = vmi->propertyValue("y")->as<rive::ViewModelInstanceNumber>();
    xProp->propertyValue(100.0f);
    yProp->propertyValue(100.0f);

    stateMachine->bindViewModelInstance(vmi);
    // Settle the initial sync — target-first precedence makes the authored
    // target values win and flow back into the source.
    stateMachine->advanceAndApply(0.0f);
    for (int i = 0; i < 10; i++)
    {
        stateMachine->advanceAndApply(0.016f);
    }

    rive::Node* targetNode = nullptr;
    for (auto* db : artboard->dataBinds())
    {
        if (db->target() != nullptr && db->target()->is<rive::Node>())
        {
            targetNode = db->target()->as<rive::Node>();
            break;
        }
    }
    REQUIRE(targetNode != nullptr);

    // After settling the source mirrors the target (they are two-way bound and
    // the target won the initial sync).
    REQUIRE(xProp->propertyValue() == targetNode->x());
    REQUIRE(yProp->propertyValue() == targetNode->y());

    // Change the SOURCE to clearly distinct values. With the fix this
    // propagates source→target only; the buggy behavior ran target→source first
    // and reverted the source to the (stale) target value.
    xProp->propertyValue(500.0f);
    yProp->propertyValue(600.0f);
    for (int i = 0; i < 20; i++)
    {
        stateMachine->advanceAndApply(0.016f);
    }

    // Source keeps its new value (not clobbered) and reaches the target.
    CHECK(xProp->propertyValue() == 500.0f);
    CHECK(yProp->propertyValue() == 600.0f);
    CHECK(targetNode->x() == 500.0f);
    CHECK(targetNode->y() == 600.0f);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 32 awaits typed Rust execution"]
fn wave_b_data_binding_test_032_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Artboards as conditions", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/databind_artboard.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi =
        file->createViewModelInstance((int)artboard.get()->viewModelId(), 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);
    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    silver.addFrame();

    stateMachine->pointerDown(rive::Vec2D(247, 332));
    stateMachine->pointerUp(rive::Vec2D(247, 332));
    stateMachine->advanceAndApply(0.1f);

    artboard->draw(renderer.get());

    silver.addFrame();

    stateMachine->advanceAndApply(0.1f);

    artboard->draw(renderer.get());

    CHECK(silver.matches("databind_artboard"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 33 awaits typed Rust execution"]
fn wave_b_data_binding_test_033_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Relative data binding", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/relative_data_binding.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi =
        file->createViewModelInstance((int)artboard.get()->viewModelId(), 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);
    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    CHECK(silver.matches("relative_data_binding"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 34 awaits typed Rust execution"]
fn wave_b_data_binding_test_034_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Relative data binding view model path", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/relative_data_bind_path.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    auto renderer = silver.makeRenderer();
    // First use the default view model instance that is attached to the
    // artboard
    {
        auto vmi =
            file->createViewModelInstance((int)artboard.get()->viewModelId(),
                                          0);

        stateMachine->bindViewModelInstance(vmi);
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
        silver.addFrame();
    }
    // Next bind it to a different view model type that matches the expected
    // view model shape
    {
        auto vm = file->viewModel("ViewModel1");
        auto vmi = file->createDefaultViewModelInstance(vm);
        stateMachine->bindViewModelInstance(vmi);
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
        silver.addFrame();
    }
    // Next bind it to a different view model with the wrong shape
    {
        auto vm = file->viewModel("ViewModel2");
        auto vmi = file->createDefaultViewModelInstance(vm);
        stateMachine->bindViewModelInstance(vmi);
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("relative_data_bind_path"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 35 awaits typed Rust execution"]
fn wave_b_data_binding_test_035_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Relative data binding view model state machine listener", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/relative_data_bind_path.riv", &silver);

    auto artboard = file->artboardNamed("listener");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    auto renderer = silver.makeRenderer();
    // First use the default view model instance that is attached to the
    // artboard
    {
        auto vmi =
            file->createViewModelInstance((int)artboard.get()->viewModelId(),
                                          0);
        auto numProp = vmi->propertyValue("num")->as<ViewModelInstanceNumber>();

        stateMachine->bindViewModelInstance(vmi);
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
        silver.addFrame();
        numProp->propertyValue(100);
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
        silver.addFrame();
    }
    // Next bind it to a different view model with the same shape
    {
        auto vm = file->viewModel("SML_VM2");
        auto vmi = file->createDefaultViewModelInstance(vm);
        auto numProp = vmi->propertyValue("num")->as<ViewModelInstanceNumber>();

        stateMachine->bindViewModelInstance(vmi);
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
        silver.addFrame();
        numProp->propertyValue(100);
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("relative_data_bind_path-listener"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 36 awaits typed Rust execution"]
fn wave_b_data_binding_test_036_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Relative data binding view model state machine fire trigger",
          "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/relative_data_bind_path.riv", &silver);

    auto artboard = file->artboardNamed("fire-trigger");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    auto renderer = silver.makeRenderer();
    // First use the default view model instance that is attached to the
    // artboard
    {
        auto vmi =
            file->createViewModelInstance((int)artboard.get()->viewModelId(),
                                          0);
        auto resetProp =
            vmi->propertyValue("reset")->as<ViewModelInstanceTrigger>();

        stateMachine->bindViewModelInstance(vmi);
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
        silver.addFrame();
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
        silver.addFrame();
        resetProp->trigger();
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
        silver.addFrame();
    }
    // Next bind it to a different view model with the same shape
    {
        auto vm = file->viewModel("SMFT-VM2");
        auto vmi = file->createDefaultViewModelInstance(vm);

        stateMachine->bindViewModelInstance(vmi);
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
        silver.addFrame();
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("relative_data_bind_path-fire-trigger"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 37 awaits typed Rust execution"]
fn wave_b_data_binding_test_037_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Relative data binding view model scripted input", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/relative_data_bind_path.riv", &silver);

    auto artboard = file->artboardNamed("scripted-input");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    auto renderer = silver.makeRenderer();
    // First use the default view model instance that is attached to the
    // artboard
    {
        auto vmi =
            file->createViewModelInstance((int)artboard.get()->viewModelId(),
                                          0);
        auto child =
            vmi->propertyValue("child")->as<ViewModelInstanceViewModel>();
        auto boo = child->referenceViewModelInstance()
                       ->propertyValue("boo")
                       ->as<ViewModelInstanceBoolean>();
        auto paused = child->referenceViewModelInstance()
                          ->propertyValue("paused")
                          ->as<ViewModelInstanceBoolean>();
        stateMachine->bindViewModelInstance(vmi);
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
        silver.addFrame();
        paused->propertyValue(false);
        stateMachine->advanceAndApply(1.0f);
        artboard->draw(renderer.get());
        silver.addFrame();
        paused->propertyValue(true);
        boo->propertyValue(false);
        stateMachine->advanceAndApply(1.0f);
        artboard->draw(renderer.get());
        silver.addFrame();
    }
    // Next bind it to a different view model with the same shape
    {
        auto vm = file->viewModel("SI-VM2");
        auto vmi = file->createDefaultViewModelInstance(vm);
        auto child =
            vmi->propertyValue("child")->as<ViewModelInstanceViewModel>();
        auto boo = child->referenceViewModelInstance()
                       ->propertyValue("boo")
                       ->as<ViewModelInstanceBoolean>();
        auto paused = child->referenceViewModelInstance()
                          ->propertyValue("paused")
                          ->as<ViewModelInstanceBoolean>();

        stateMachine->bindViewModelInstance(vmi);
        stateMachine->advanceAndApply(0.1f);
        artboard->draw(renderer.get());
        silver.addFrame();
        paused->propertyValue(false);
        stateMachine->advanceAndApply(1.0f);
        artboard->draw(renderer.get());
        silver.addFrame();
        paused->propertyValue(true);
        boo->propertyValue(false);
        stateMachine->advanceAndApply(1.0f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("relative_data_bind_path-scripted-input"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 38 awaits typed Rust execution"]
fn wave_b_data_binding_test_038_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Listen to view model value changes in state machines", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/listener_view_model.riv", &silver);

    auto artboard = file->artboardDefault();
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);
    auto colorProp = vmi->propertyValue("col")->as<ViewModelInstanceColor>();
    auto triggerProp =
        vmi->propertyValue("tri")->as<ViewModelInstanceTrigger>();
    auto numProp = vmi->propertyValue("num1")->as<ViewModelInstanceNumber>();

    stateMachine->bindViewModelInstance(vmi);
    auto renderer = silver.makeRenderer();
    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());
    silver.addFrame();
    colorProp->propertyValue(rive::colorARGB(100, 0, 10, 15));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    triggerProp->trigger();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    numProp->propertyValue(55);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("listener_view_model"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 39 awaits typed Rust execution"]
fn wave_b_data_binding_test_039_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Artboard properties conditions work without binding", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/artboard_width_test.riv", &silver);

    auto artboard = file->artboardDefault();
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);

    auto renderer = silver.makeRenderer();
    stateMachine->advanceAndApply(0.0f);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("artboard_width_test"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_test case 40 awaits typed Rust execution"]
fn wave_b_data_binding_test_040_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("TwoWay target change reaches source under source-first precedence",
          "[data binding]")
{
    auto file = ReadRiveFile("assets/bidirectional_precedence.riv");
    auto artboard = file->artboardNamed("source_first");
    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();
    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    auto xProp = vmi->propertyValue("x")->as<rive::ViewModelInstanceNumber>();
    auto yProp = vmi->propertyValue("y")->as<rive::ViewModelInstanceNumber>();
    // Source values differ from the authored target; source-first precedence
    // makes the source win on the initial sync, so the source value (100) is
    // what a spurious source→target apply would clobber the target with.
    xProp->propertyValue(100.0f);
    yProp->propertyValue(100.0f);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);

    // The two TwoWay binds target an (unnamed) node's x/y — reach it through a
    // bind rather than by name.
    rive::Node* targetNode = nullptr;
    for (auto* db : artboard->dataBinds())
    {
        if (db->target() != nullptr && db->target()->is<rive::Node>())
        {
            targetNode = db->target()->as<rive::Node>();
            break;
        }
    }
    REQUIRE(targetNode != nullptr);

    // Change the TARGET directly. This is the trigger that regressed: with the
    // fix it propagates target→source; without it the source→target apply
    // overwrites these with the source value (100) first.
    targetNode->x(700.0f);
    targetNode->y(800.0f);

    stateMachine->advanceAndApply(0.016f);

    CHECK(xProp->propertyValue() == 700.0f);
    CHECK(yProp->propertyValue() == 800.0f);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_viewmodels_test case 1 awaits typed Rust execution"]
fn wave_b_data_binding_viewmodels_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE(
    "Data bind view model to view model instance from set value, externally and from scripting",
    "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/databind_viewmodel.riv", &silver);

    auto artboard = file->artboardDefault();
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    auto vmi = file->createDefaultViewModelInstance(artboard.get());

    stateMachine->bindViewModelInstance(vmi);
    auto renderer = silver.makeRenderer();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    auto statefulChildVMI = file->createViewModelInstance("StatefulChild");
    auto numProp =
        statefulChildVMI->propertyValue("num")->as<ViewModelInstanceNumber>();
    numProp->propertyValue(44.0f);

    vmi->replaceViewModelByName("statefulChild", statefulChildVMI);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    numProp->propertyValue(44.0f);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    stateMachine->pointerDown(Vec2D(25.0f, 25.0f));
    stateMachine->pointerUp(Vec2D(25.0f, 25.0f));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("databind_viewmodel"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_viewmodels_test case 2 awaits typed Rust execution"]
fn wave_b_data_binding_viewmodels_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Stateful component is bound before binding the view model instance",
          "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/unbound_stateful_component.riv", &silver);

    auto artboard = file->artboardDefault();
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    auto vmi = file->createDefaultViewModelInstance(artboard.get());

    auto renderer = silver.makeRenderer();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("unbound_stateful_component"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned data_binding_viewmodels_test case 3 awaits typed Rust execution"]
fn wave_b_data_binding_viewmodels_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Bidirectional stateful property binding", "[silver]")
{
    SerializingFactory silver;
    auto file =
        ReadRiveFile("assets/bidirectional_stateful_property.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);
    auto vmi = file->createDefaultViewModelInstance(artboard.get());

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);
    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(175, 175));
    stateMachine->pointerUp(rive::Vec2D(175, 175));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(450, 450));
    stateMachine->pointerUp(rive::Vec2D(450, 450));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(175, 175));
    stateMachine->pointerUp(rive::Vec2D(175, 175));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(450, 450));
    stateMachine->pointerUp(rive::Vec2D(450, 450));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(450, 50));
    stateMachine->pointerUp(rive::Vec2D(450, 50));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    int frames = (int)(1.0f / 0.2f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.2f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("bidirectional_stateful_property"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned decode_ktx2_test case 1 awaits typed Rust execution"]
fn wave_b_decode_ktx2_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ktx2 rejects buffer smaller than identifier+header",
          "[ktx2-decoder]")
{
    std::vector<uint8_t> buf(40, 0);
    rive::Ktx2DecodeResult out;
    REQUIRE_FALSE(rive::DecodeKtx2(buf.data(), buf.size(), out));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned decode_ktx2_test case 2 awaits typed Rust execution"]
fn wave_b_decode_ktx2_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ktx2 rejects bad magic", "[ktx2-decoder]")
{
    auto buf = buildSkeletonKtx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 4, 1, 0, 1, 0);
    buf[0] = 'X';
    rive::Ktx2DecodeResult out;
    REQUIRE_FALSE(rive::DecodeKtx2(buf.data(), buf.size(), out));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned decode_ktx2_test case 3 awaits typed Rust execution"]
fn wave_b_decode_ktx2_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ktx2 rejects unsupported vkFormat", "[ktx2-decoder]")
{
    // VK_FORMAT_R8G8B8A8_UNORM = 37 — not BC7.
    auto buf = buildSkeletonKtx2(/*vkFormat*/ 37, 4, 4, 1, 0, 1, 0);
    rive::Ktx2DecodeResult out;
    REQUIRE_FALSE(rive::DecodeKtx2(buf.data(), buf.size(), out));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned decode_ktx2_test case 4 awaits typed Rust execution"]
fn wave_b_decode_ktx2_test_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ktx2 rejects supercompressed payload (not yet supported)",
          "[ktx2-decoder]")
{
    auto buf = buildSkeletonKtx2(VK_FORMAT_BC7_SRGB_BLOCK,
                                 4,
                                 4,
                                 1,
                                 /*supercompressionScheme*/ 2 /* zstd */,
                                 1,
                                 0);
    rive::Ktx2DecodeResult out;
    REQUIRE_FALSE(rive::DecodeKtx2(buf.data(), buf.size(), out));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned decode_ktx2_test case 5 awaits typed Rust execution"]
fn wave_b_decode_ktx2_test_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ktx2 rejects cubemaps and array layers", "[ktx2-decoder]")
{
    {
        auto buf = buildSkeletonKtx2(VK_FORMAT_BC7_SRGB_BLOCK,
                                     4,
                                     4,
                                     1,
                                     0,
                                     /*faceCount*/ 6,
                                     0);
        rive::Ktx2DecodeResult out;
        REQUIRE_FALSE(rive::DecodeKtx2(buf.data(), buf.size(), out));
    }
    {
        auto buf = buildSkeletonKtx2(VK_FORMAT_BC7_SRGB_BLOCK,
                                     4,
                                     4,
                                     1,
                                     0,
                                     1,
                                     /*layerCount*/ 4);
        rive::Ktx2DecodeResult out;
        REQUIRE_FALSE(rive::DecodeKtx2(buf.data(), buf.size(), out));
    }
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned decode_ktx2_test case 6 awaits typed Rust execution"]
fn wave_b_decode_ktx2_test_006_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ktx2 rejects out-of-range dimensions", "[ktx2-decoder]")
{
    {
        auto buf =
            buildSkeletonKtx2(VK_FORMAT_BC7_SRGB_BLOCK, 0, 4, 1, 0, 1, 0);
        rive::Ktx2DecodeResult out;
        REQUIRE_FALSE(rive::DecodeKtx2(buf.data(), buf.size(), out));
    }
    {
        auto buf =
            buildSkeletonKtx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 999999, 1, 0, 1, 0);
        rive::Ktx2DecodeResult out;
        REQUIRE_FALSE(rive::DecodeKtx2(buf.data(), buf.size(), out));
    }
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned decode_ktx2_test case 7 awaits typed Rust execution"]
fn wave_b_decode_ktx2_test_007_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ktx2 rejects truncated level index", "[ktx2-decoder]")
{
    auto buf = buildSkeletonKtx2(VK_FORMAT_BC7_SRGB_BLOCK,
                                 4,
                                 4,
                                 /*levelCount*/ 1,
                                 0,
                                 1,
                                 0);
    rive::Ktx2DecodeResult out;
    REQUIRE_FALSE(rive::DecodeKtx2(buf.data(), buf.size(), out));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned decode_ktx2_test case 8 awaits typed Rust execution"]
fn wave_b_decode_ktx2_test_008_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ktx2 rejects level pointer outside buffer", "[ktx2-decoder]")
{
    auto buf = buildSkeletonKtx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 4, 1, 0, 1, 0);
    appendLE<uint64_t>(buf, /*byteOffset*/ 1ull << 32);
    appendLE<uint64_t>(buf, /*byteLength*/ 16);
    appendLE<uint64_t>(buf, /*uncompressedByteLength*/ 16);
    rive::Ktx2DecodeResult out;
    REQUIRE_FALSE(rive::DecodeKtx2(buf.data(), buf.size(), out));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned decode_ktx2_test case 9 awaits typed Rust execution"]
fn wave_b_decode_ktx2_test_009_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ktx2 rejects byteLength inconsistent with logical block grid",
          "[ktx2-decoder]")
{
    // 4x4 image = 1 BC7 block = 16 bytes. Claiming 32 bytes mismatches.
    auto buf = buildSkeletonKtx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 4, 1, 0, 1, 0);
    const uint64_t levelOffset = buf.size() + 24;
    appendLE<uint64_t>(buf, levelOffset);
    appendLE<uint64_t>(buf, /*byteLength*/ 32);
    appendLE<uint64_t>(buf, 32);
    buf.resize(buf.size() + 32, 0);
    rive::Ktx2DecodeResult out;
    REQUIRE_FALSE(rive::DecodeKtx2(buf.data(), buf.size(), out));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned decode_ktx2_test case 10 awaits typed Rust execution"]
fn wave_b_decode_ktx2_test_010_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ktx2 happy path: single 4x4 BC7 mip 0", "[ktx2-decoder]")
{
    auto buf = buildSkeletonKtx2(VK_FORMAT_BC7_SRGB_BLOCK, 4, 4, 1, 0, 1, 0);
    const uint64_t levelOffset = buf.size() + 24;
    appendLE<uint64_t>(buf, levelOffset);
    appendLE<uint64_t>(buf, /*byteLength*/ 16);
    appendLE<uint64_t>(buf, /*uncompressedByteLength*/ 16);
    // 16 bytes of synthetic block payload — parser doesn't validate the
    // BC7 bitstream, just copies the bytes through.
    const uint8_t expected[16] = {
        0xDE,
        0xAD,
        0xBE,
        0xEF,
        0x01,
        0x02,
        0x03,
        0x04,
        0x05,
        0x06,
        0x07,
        0x08,
        0xCA,
        0xFE,
        0xBA,
        0xBE,
    };
    buf.insert(buf.end(), expected, expected + 16);

    rive::Ktx2DecodeResult out;
    REQUIRE(rive::DecodeKtx2(buf.data(), buf.size(), out));
    REQUIRE(out.format == rive::GPUTextureFormat::bc7);
    REQUIRE(out.pixelWidth == 4);
    REQUIRE(out.pixelHeight == 4);
    REQUIRE(out.levelCount == 1);
    REQUIRE(out.blocks.size() == 16);
    REQUIRE(std::memcmp(out.blocks.data(), expected, 16) == 0);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned decode_ktx2_test case 11 awaits typed Rust execution"]
fn wave_b_decode_ktx2_test_011_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ktx2 happy path: 8x8 with two mip levels concatenated",
          "[ktx2-decoder]")
{
    // Mip 0 = 8x8 = 4 blocks (64 bytes). Mip 1 = 4x4 = 1 block (16 bytes).
    // Level index lists level 0 first; on disk levels should sit smallest-
    // first but the parser only reads each level by its own offset, so
    // ordering within the buffer doesn't matter here.
    auto buf = buildSkeletonKtx2(VK_FORMAT_BC7_SRGB_BLOCK, 8, 8, 2, 0, 1, 0);

    const uint64_t headerEnd = buf.size();
    const uint64_t levelIndexBytes = 24 * 2;
    const uint64_t mip0Offset = headerEnd + levelIndexBytes;
    const uint64_t mip0Bytes = 64;
    const uint64_t mip1Offset = mip0Offset + mip0Bytes;
    const uint64_t mip1Bytes = 16;

    // Level index entry 0 (mip 0, 8x8).
    appendLE<uint64_t>(buf, mip0Offset);
    appendLE<uint64_t>(buf, mip0Bytes);
    appendLE<uint64_t>(buf, mip0Bytes);
    // Level index entry 1 (mip 1, 4x4).
    appendLE<uint64_t>(buf, mip1Offset);
    appendLE<uint64_t>(buf, mip1Bytes);
    appendLE<uint64_t>(buf, mip1Bytes);

    // Block payloads. Distinct fill bytes so the test can verify ordering.
    buf.resize(buf.size() + mip0Bytes, 0xAA);
    buf.resize(buf.size() + mip1Bytes, 0xBB);

    rive::Ktx2DecodeResult out;
    REQUIRE(rive::DecodeKtx2(buf.data(), buf.size(), out));
    REQUIRE(out.pixelWidth == 8);
    REQUIRE(out.pixelHeight == 8);
    REQUIRE(out.levelCount == 2);
    REQUIRE(out.blocks.size() == mip0Bytes + mip1Bytes);
    // Output buffer is concatenated level 0 (largest) first, then level 1.
    REQUIRE(out.blocks[0] == 0xAA);
    REQUIRE(out.blocks[mip0Bytes - 1] == 0xAA);
    REQUIRE(out.blocks[mip0Bytes] == 0xBB);
    REQUIRE(out.blocks[mip0Bytes + mip1Bytes - 1] == 0xBB);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned default_state_machine_test case 1 awaits typed Rust execution"]
fn wave_b_default_state_machine_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("default state machine is detected at load", "[file]")
{
    auto file = ReadRiveFile("assets/entry.riv");

    auto abi = file->artboardAt(0);
    auto index = abi->defaultStateMachineIndex();

    REQUIRE(index >= 0);
    REQUIRE(abi->stateMachineNameAt(index) == "State Machine 1");

    auto smi = abi->defaultStateMachine();

    REQUIRE(smi != nullptr);
    REQUIRE(smi->name() == "State Machine 1");

    // default scene is the same as the default statemachine (when we have one)
    auto scene = abi->defaultScene();
    REQUIRE(scene != nullptr);
    REQUIRE(scene->name() == smi->name());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned distance_constraint_test case 1 awaits typed Rust execution"]
fn wave_b_distance_constraint_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("distance constraints moves items as expected", "[file]")
{
    auto file = ReadRiveFile("assets/distance_constraint.riv");

    auto artboard = file->artboard();

    REQUIRE(artboard->find<rive::Shape>("A") != nullptr);
    auto a = artboard->find<rive::Shape>("A");

    REQUIRE(artboard->find<rive::Shape>("B") != nullptr);
    auto b = artboard->find<rive::Shape>("B");

    REQUIRE(a->constraints().size() == 1);
    REQUIRE(a->constraints()[0]->is<rive::DistanceConstraint>());

    auto distanceConstraint =
        a->constraints()[0]->as<rive::DistanceConstraint>();
    REQUIRE(distanceConstraint->modeValue() == 1);

    b->x(259.31f);
    b->y(137.87f);
    artboard->advance(0.0f);

    rive::Vec2D at = a->worldTranslation();
    rive::Vec2D expectedTranslation(259.2808837890625f, 62.87000274658203f);
    REQUIRE(rive::Vec2D::distance(at, expectedTranslation) < 0.001f);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned draw_order_test case 1 awaits typed Rust execution"]
fn wave_b_draw_order_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("draw rules load and sort correctly", "[draw rules]")
{
    auto file = ReadRiveFile("assets/draw_rule_cycle.riv");

    // auto file = reader.file();
    std::unique_ptr<rive::ArtboardInstance> artboard = file->artboardDefault();
    auto node = artboard->find<rive::Node>("Blue");
    REQUIRE(node != nullptr);
    REQUIRE(node->is<rive::Shape>());
    // auto shape = node->as<rive::Shape>();

    artboard->updateComponents();
    REQUIRE(artboard->animationCount() == 1);

    // Check that we can advance the ping-pong animation with 1 second duration
    // without a hang.
    std::unique_ptr<rive::LinearAnimationInstance> animation =
        artboard->animationAt(0);
    // Advance and apply some frames.
    int frames = 10;
    float frameDuration = 1.0f;

    for (int i = 0; i < frames; i++)
    {
        animation->advanceAndApply(frameDuration);
        rive::NoOpRenderer renderer;
        artboard->draw(&renderer);
    }
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned elastic_easing_test case 1 awaits typed Rust execution"]
fn wave_b_elastic_easing_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("test elastic easing loads properly", "[file]")
{
    auto file = ReadRiveFile("assets/test_elastic.riv");

    auto artboard = file->artboard();
    REQUIRE(artboard != nullptr);

    REQUIRE(artboard->find<rive::ElasticInterpolator>().size() == 1);

    auto interpolator = artboard->find<rive::ElasticInterpolator>()[0];
    REQUIRE(interpolator->easing() == rive::Easing::easeOut);
    REQUIRE(interpolator->amplitude() == 1.0f);
    REQUIRE(interpolator->period() == 0.25f);

    REQUIRE(artboard->find<rive::Shape>().size() == 1);

    auto shape = artboard->find<rive::Shape>()[0];
    REQUIRE(shape->x() == Approx(145.19f));
    auto animation = artboard->animation("Timeline 1");
    REQUIRE(animation != nullptr);
    // Go to frame 15.
    animation->apply(artboard, 7.0f / animation->fps(), 1.0f);
    REQUIRE(shape->x() == Approx(423.98f));

    animation->apply(artboard, 14.0f / animation->fps(), 1.0f);
    REQUIRE(shape->x() == Approx(303.995f));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned elastic_easing_test case 2 awaits typed Rust execution"]
fn wave_b_elastic_easing_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("elastic easer computes correct actual amplitude", "[animation]")
{
    rive::ElasticEase easer(0.5f, 3.14f);
    REQUIRE(easer.computeActualAmplitude(0.0f) == 1.0f);
    REQUIRE(easer.computeActualAmplitude(1.57f) == 0.5f);
    REQUIRE(easer.easeOut(0.22f) == Catch::Detail::Approx(0.8307f));
    REQUIRE(easer.easeIn(1.58f) == Catch::Detail::Approx(14.01086f));
    REQUIRE(easer.easeInOut(1.58f) == Catch::Detail::Approx(1.0f));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 1 awaits typed Rust execution"]
fn wave_b_enums_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("flag operator|", "[enums]")
{
    TestBinaryEnumOp<Flags>([](auto a, auto b) { return a | b; });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 2 awaits typed Rust execution"]
fn wave_b_enums_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("flag operator&", "[enums]")
{
    TestBinaryEnumOp<Flags>([](auto a, auto b) { return a & b; });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 3 awaits typed Rust execution"]
fn wave_b_enums_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("flag operator^", "[enums]")
{
    TestBinaryEnumOp<Flags>([](auto a, auto b) { return a ^ b; });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 4 awaits typed Rust execution"]
fn wave_b_enums_test_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("flag operator~", "[enums]")
{
    TestUnaryEnumOp<Flags>([](auto a) { return ~a; });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 5 awaits typed Rust execution"]
fn wave_b_enums_test_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("flag operator |=", "[enums]")
{
    TestBinaryEnumOp<Flags>([](auto a, auto b) {
        auto r = a;
        r |= b;
        return r;
    });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 6 awaits typed Rust execution"]
fn wave_b_enums_test_006_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("flag operator &=", "[enums]")
{
    TestBinaryEnumOp<Flags>([](auto a, auto b) {
        auto r = a;
        r &= b;
        return r;
    });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 7 awaits typed Rust execution"]
fn wave_b_enums_test_007_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("flag operator ^=", "[enums]")
{
    TestBinaryEnumOp<Flags>([](auto a, auto b) {
        auto r = a;
        r ^= b;
        return r;
    });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 8 awaits typed Rust execution"]
fn wave_b_enums_test_008_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("is_single_flag", "[enums]")
{
    CHECK(!is_single_flag(Flags::none));
    CHECK(!is_single_flag(Flags64::none));
    for (auto i = 0u; i < 32; i++)
    {
        CHECK(is_single_flag(Flags(1u << i)));
    }

    for (auto i = 0u; i < 64; i++)
    {
        CHECK(is_single_flag(Flags64(uint64_t(1u) << i)));
    }

    TestUnaryEnumOp<Flags>([](auto a) { return is_single_flag(a); });
    TestUnaryEnumOp<Flags64>([](auto a) { return is_single_flag(a); });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 9 awaits typed Rust execution"]
fn wave_b_enums_test_009_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("is_flag_set", "[enums]")
{
    // Because the second parameter needs to be a single flag, we'll do this as
    // a bespoke test instead of using TestBinaryEnumOp
    auto doTest = [](auto unusedEnumValue) {
        // the parameter is unused, except to give us the enum type to test.
        std::ignore = unusedEnumValue;

        using Enum = decltype(unusedEnumValue);
        using U = std::underlying_type_t<Enum>;

        CHECK(!is_flag_set(Enum::none, Enum::one));
        CHECK(is_flag_set(Enum::one, Enum::one));
        CHECK(!is_flag_set(Enum::one, Enum::two));
        CHECK(is_flag_set(Enum::one | Enum::two, Enum::one));

        auto seed = 0xf934929u; // arbitrary, but consistent, seed
        std::mt19937_64 random{seed};

        constexpr auto FLAG_BIT_COUNT = sizeof(U) * CHAR_BIT;
        constexpr auto TEST_COUNT_PER_FLAG_BIT = 100u;
        for (auto bitIndex = 0u; bitIndex < FLAG_BIT_COUNT; bitIndex++)
        {
            auto iTest = U(1) << bitIndex;
            auto eTest = Enum(iTest);
            CHECK(!is_flag_set(Enum::none, eTest));

            for (auto testIndex = 0u; testIndex < TEST_COUNT_PER_FLAG_BIT;
                 testIndex++)
            {
                auto iValue = U(random());
                auto eValue = Enum(iValue);
                CHECK(is_flag_set(eValue, eTest) == ((iValue & iTest) != 0));
            }
        }
    };

    doTest(Flags{});
    doTest(Flags64{});
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 10 awaits typed Rust execution"]
fn wave_b_enums_test_010_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("underlying_value", "[enums]")
{
    TestUnaryEnumOp<Flags>([](auto a) { return underlying_value(a); });
    TestUnaryEnumOp<Flags64>([](auto a) { return underlying_value(a); });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 11 awaits typed Rust execution"]
fn wave_b_enums_test_011_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("incr", "[enums]")
{
    TestUnaryEnumOp<Flags>([](auto a) { return incr(a); });
    TestUnaryEnumOp<Flags64>([](auto a) { return incr(a); });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 12 awaits typed Rust execution"]
fn wave_b_enums_test_012_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("decr", "[enums]")
{
    TestUnaryEnumOp<Flags>([](auto a) { return decr(a); });
    TestUnaryEnumOp<Flags64>([](auto a) { return decr(a); });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 13 awaits typed Rust execution"]
fn wave_b_enums_test_013_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("any_flag_set (unmasked)", "[enums]")
{
    TestUnaryEnumOp<Flags>([](auto a) { return any_flag_set(a); });
    TestUnaryEnumOp<Flags64>([](auto a) { return decr(a); });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 14 awaits typed Rust execution"]
fn wave_b_enums_test_014_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("any_flag_set (masked)", "[enums]")
{
    TestBinaryEnumOp<Flags>([](auto a, auto b) { return any_flag_set(a, b); });
    TestBinaryEnumOp<Flags64>(
        [](auto a, auto b) { return any_flag_set(a, b); });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 15 awaits typed Rust execution"]
fn wave_b_enums_test_015_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("all_flags_set", "[enums]")
{
    TestBinaryEnumOp<Flags>([](auto a, auto b) { return all_flags_set(a, b); });
    TestBinaryEnumOp<Flags64>(
        [](auto a, auto b) { return all_flags_set(a, b); });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 16 awaits typed Rust execution"]
fn wave_b_enums_test_016_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("no_flag_set (unmasked)", "[enums]")
{
    TestUnaryEnumOp<Flags>([](auto a) { return no_flags_set(a); });
    TestUnaryEnumOp<Flags64>([](auto a) { return no_flags_set(a); });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned enums_test case 17 awaits typed Rust execution"]
fn wave_b_enums_test_017_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("no_flags_set (masked)", "[enums]")
{
    TestBinaryEnumOp<Flags>([](auto a, auto b) { return no_flags_set(a, b); });
    TestBinaryEnumOp<Flags64>(
        [](auto a, auto b) { return no_flags_set(a, b); });
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned file_test case 1 awaits typed Rust execution"]
fn wave_b_file_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("transform order is as expected", "[transform]")
{
    auto translation = rive::Mat2D::fromTranslate(10.0f, 20.0f);
    auto rotation = rive::Mat2D::fromRotation(3.14f / 2.0f);
    auto scale = rive::Mat2D::fromScale(2.0f, 3.0f);

    auto xform = translation * rotation * scale;
    auto xform2 = rive::Mat2D::fromRotation(3.14f / 2.0f);
    xform2[0] *= 2.0f;
    xform2[1] *= 2.0f;
    xform2[2] *= 3.0f;
    xform2[3] *= 3.0f;
    xform2[4] = 10.0f;
    xform2[5] = 20.0f;

    REQUIRE(xform2 == xform);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned file_test case 2 awaits typed Rust execution"]
fn wave_b_file_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("file can be read", "[file]")
{
    auto file = ReadRiveFile("assets/two_artboards.riv");

    // Default artboard should be named Two.
    REQUIRE(file->artboard()->name() == "Two");

    // There should be a second artboard named One.
    REQUIRE(file->artboard("One") != nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned file_test case 3 awaits typed Rust execution"]
fn wave_b_file_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("file with bad blend mode fails to load", "[file]")
{
    std::vector<uint8_t> bytes = ReadFile("assets/solar-system.riv");

    rive::ImportResult result;
    auto file = rive::File::import(bytes, &gNoOpFactory, &result, nullptr);
    CHECK(result == rive::ImportResult::malformed);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned file_test case 4 awaits typed Rust execution"]
fn wave_b_file_test_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("file with animation can be read", "[file]")
{
    auto file = ReadRiveFile("assets/juice.riv");

    auto artboard = file->artboard();
    REQUIRE(artboard->name() == "New Artboard");

    auto shin = artboard->find("shin_right");
    REQUIRE(shin != nullptr);
    REQUIRE(shin->is<rive::Node>());

    auto shinNode = shin->as<rive::Node>();
    REQUIRE(shinNode->parent() != nullptr);
    REQUIRE(shinNode->parent()->name() == "leg_right");
    REQUIRE(shinNode->parent()->parent() != nullptr);
    REQUIRE(shinNode->parent()->parent()->name() == "root");
    REQUIRE(shinNode->parent()->parent() != nullptr);
    REQUIRE(shinNode->parent()->parent()->parent() != nullptr);
    REQUIRE(shinNode->parent()->parent()->parent() == artboard);

    auto walkAnimation = artboard->animation("walk");
    REQUIRE(walkAnimation != nullptr);
    REQUIRE(walkAnimation->numKeyedObjects() == 22);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned file_test case 5 awaits typed Rust execution"]
fn wave_b_file_test_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("artboards can be counted and accessed via index or name", "[file]")
{
    auto file = ReadRiveFile("assets/dependency_test.riv");

    // The artboards caqn be counted
    REQUIRE(file->artboardCount() == 1);

    // Artboards can be access by index
    REQUIRE(file->artboard(0) != nullptr);

    // Artboards can be accessed by name
    REQUIRE(file->artboard("Blue") != nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned file_test case 6 awaits typed Rust execution"]
fn wave_b_file_test_006_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("dependencies are as expected", "[file]")
{
    // ┌────┐
    // │Blue│
    // └────┘
    //    │ ┌───┐
    //    └▶│ A │
    //      └───┘
    //        │ ┌───┐
    //        └▶│ B │
    //          └───┘
    //            │ ┌───┐
    //            ├▶│ C │
    //            │ └───┘
    //            │ ┌─────────┐
    //            └▶│Rectangle│
    //              └─────────┘
    //                   │ ┌──────────────┐
    //                   └▶│Rectangle Path│
    //                     └──────────────┘
    auto file = ReadRiveFile("assets/dependency_test.riv");

    auto artboard = file->artboard();
    REQUIRE(artboard->name() == "Blue");

    auto nodeA = artboard->find<rive::Node>("A");
    auto nodeB = artboard->find<rive::Node>("B");
    auto nodeC = artboard->find<rive::Node>("C");
    auto shape = artboard->find<rive::Shape>("Rectangle");
    auto path = artboard->find<rive::Path>("Rectangle Path");
    REQUIRE(nodeA != nullptr);
    REQUIRE(nodeB != nullptr);
    REQUIRE(nodeC != nullptr);
    REQUIRE(shape != nullptr);
    REQUIRE(path != nullptr);

    REQUIRE(nodeA->parent() == artboard);
    REQUIRE(nodeB->parent() == nodeA);
    REQUIRE(nodeC->parent() == nodeB);
    REQUIRE(shape->parent() == nodeB);
    REQUIRE(path->parent() == shape);

    REQUIRE(nodeB->dependents().size() == 2);

    REQUIRE(artboard->graphOrder() == 0);
    REQUIRE(nodeA->graphOrder() > artboard->graphOrder());
    REQUIRE(nodeB->graphOrder() > nodeA->graphOrder());
    REQUIRE(nodeC->graphOrder() > nodeB->graphOrder());
    REQUIRE(shape->graphOrder() > nodeB->graphOrder());
    REQUIRE(path->graphOrder() > shape->graphOrder());

    artboard->advance(0.0f);

    auto world = shape->worldTransform();
    REQUIRE(world[4] == 39.203125f);
    REQUIRE(world[5] == 29.535156f);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned file_test case 7 awaits typed Rust execution"]
fn wave_b_file_test_007_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("long name in object is parsed correctly", "[file]")
{
    auto file = ReadRiveFile("assets/long_name.riv");
    auto artboard = file->artboard();

    // Expect all object in file to be loaded, in this case 7
    REQUIRE(artboard->objects().size() == 7);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned file_test case 8 awaits typed Rust execution"]
fn wave_b_file_test_008_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("file with in-band images can have the stripped", "[file]")
{
    FILE* fp = fopen("assets/jellyfish_test.riv", "rb");
    REQUIRE(fp != nullptr);

    fseek(fp, 0, SEEK_END);
    const size_t length = ftell(fp);
    fseek(fp, 0, SEEK_SET);
    std::vector<uint8_t> bytes(length);
    REQUIRE(fread(bytes.data(), 1, length, fp) == length);
    fclose(fp);

    rive::ImportResult result;
    auto file = rive::File::import(bytes, &gNoOpFactory, &result);
    REQUIRE(result == rive::ImportResult::success);
    REQUIRE(file.get() != nullptr);
    REQUIRE(file->artboard() != nullptr);

    // Default artboard should be named Two.
    REQUIRE(file->artboard()->name() == "Jellyfish");

    // Strip nothing should result in the same file.
    {
        rive::ImportResult stripResult;
        auto strippedBytes = rive::File::stripAssets(bytes, {}, &stripResult);
        REQUIRE(stripResult == rive::ImportResult::success);
        REQUIRE(bytes.size() == strippedBytes.size());
        REQUIRE(std::memcmp(bytes.data(), strippedBytes.data(), bytes.size()) ==
                0);
    }

    // Strip image assets should result in a smaller file.
    {
        rive::ImportResult stripResult;
        auto strippedBytes =
            rive::File::stripAssets(bytes,
                                    {rive::ImageAsset::typeKey},
                                    &stripResult);
        REQUIRE(stripResult == rive::ImportResult::success);
        REQUIRE(strippedBytes.size() < bytes.size());
    }
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned file_test case 9 awaits typed Rust execution"]
fn wave_b_file_test_009_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("file a bad skin (no parent skinnable) doesn't crash", "[file]")
{
    FILE* fp = fopen("assets/bad_skin.riv", "rb");
    REQUIRE(fp != nullptr);

    fseek(fp, 0, SEEK_END);
    const size_t length = ftell(fp);
    fseek(fp, 0, SEEK_SET);
    std::vector<uint8_t> bytes(length);
    REQUIRE(fread(bytes.data(), 1, length, fp) == length);
    fclose(fp);

    rive::ImportResult result;
    auto file = rive::File::import(bytes, &gNoOpFactory, &result);
    REQUIRE(result == rive::ImportResult::success);
    REQUIRE(file.get() != nullptr);
    REQUIRE(file->artboard() != nullptr);

    REQUIRE(file->artboard()->name() == "Illustration WOman.svg");
    auto artboard = file->artboardDefault();
    artboard->updateComponents();
    auto paths = artboard->find<rive::PointsPath>();
    for (auto path : paths)
    {
        path->markPathDirty();
    }
    artboard->updateComponents();
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned file_test case 10 awaits typed Rust execution"]
fn wave_b_file_test_010_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("file with bad keyed property loads", "[file]")
{
    FILE* fp = fopen("assets/magic_alley_db_reduced_export.riv", "rb");
    REQUIRE(fp != nullptr);

    fseek(fp, 0, SEEK_END);
    const size_t length = ftell(fp);
    fseek(fp, 0, SEEK_SET);
    std::vector<uint8_t> bytes(length);
    REQUIRE(fread(bytes.data(), 1, length, fp) == length);
    fclose(fp);

    rive::ImportResult result;
    auto file = rive::File::import(bytes, &gNoOpFactory, &result);
    REQUIRE(result == rive::ImportResult::success);
    REQUIRE(file.get() != nullptr);
    REQUIRE(file->artboard() != nullptr);

    REQUIRE(file->artboard()->name() == "Artboard");
    auto artboard = file->artboardDefault();
    artboard->updateComponents();
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned file_test case 11 awaits typed Rust execution"]
fn wave_b_file_test_011_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("file can be read with verified signed scripts", "[file]")
{
    auto file = ReadRiveFile("assets/joel_signed.riv");

    for (auto asset : file->assets())
    {
        if (asset->is<rive::ScriptAsset>())
        {
            // All script assets should've been verified by the time the file is
            // loaded.
            CHECK(asset->as<rive::ScriptAsset>()->verified());
        }
    }
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned file_test case 12 awaits typed Rust execution"]
fn wave_b_file_test_012_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE(
    "Test deterministic mode for randomization and elastic scroll physics",
    "[file]")
{
    rive::SerializingFactory silver;
    rive::File::deterministicMode = true;
    auto file = ReadRiveFile("assets/deterministic_mode.riv", &silver);

    auto artboard = file->artboardDefault();
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());

    stateMachine->bindViewModelInstance(vmi);
    auto renderer = silver.makeRenderer();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    stateMachine->pointerDown(rive::Vec2D(artboard->width() / 2.0f, 400.0f));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    int frames = (int)(0.25f / 0.016f);
    float yPos = 400.0f;
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(artboard->width() / 2.0f, yPos),
                                  0.016f);
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
        yPos -= 40.0f;
    }
    silver.addFrame();
    stateMachine->pointerMove(rive::Vec2D(artboard->width() / 2.0f, yPos),
                              0.016f);
    stateMachine->pointerUp(rive::Vec2D(artboard->width() / 2.0f, yPos));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    frames = (int)(1.0f / 0.016f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("deterministic_mode"));
    rive::File::deterministicMode = false;
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 1 awaits typed Rust execution"]
fn wave_b_focus_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusNode default properties", "[FocusNode]")
{
    auto node = make_rcp<FocusNode>();

    CHECK(node->canFocus() == true);
    CHECK(node->canTouch() == true);
    CHECK(node->canTraverse() == true);
    CHECK(node->tabIndex() == 0);
    CHECK(node->edgeBehavior() == EdgeBehavior::parentScope);
    CHECK(node->focusable() == nullptr);
    CHECK(node->parent() == nullptr);
    CHECK(node->children().empty());
    CHECK(node->isScope() == false);
    CHECK(node->hasFocus() == false);
    CHECK(node->manager() == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 2 awaits typed Rust execution"]
fn wave_b_focus_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusNode property setters", "[FocusNode]")
{
    auto node = make_rcp<FocusNode>();

    node->canFocus(false);
    CHECK(node->canFocus() == false);

    node->canTouch(false);
    CHECK(node->canTouch() == false);

    node->canTraverse(false);
    CHECK(node->canTraverse() == false);

    node->tabIndex(42);
    CHECK(node->tabIndex() == 42);

    node->edgeBehavior(EdgeBehavior::closedLoop);
    CHECK(node->edgeBehavior() == EdgeBehavior::closedLoop);

    node->edgeBehavior(EdgeBehavior::stop);
    CHECK(node->edgeBehavior() == EdgeBehavior::stop);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 3 awaits typed Rust execution"]
fn wave_b_focus_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusNode with Focusable", "[FocusNode]")
{
    MockFocusable focusable;
    auto node = make_rcp<FocusNode>(&focusable);

    CHECK(node->focusable() == &focusable);

    // Test input delegation
    node->keyInput(Key::a, KeyModifiers::none, true, false);
    CHECK(focusable.keyInputCount == 1);
    CHECK(focusable.lastKey == Key::a);

    node->textInput("hello");
    CHECK(focusable.textInputCount == 1);
    CHECK(focusable.lastText == "hello");

    // Test lifecycle delegation
    node->focused();
    CHECK(focusable.focusedCount == 1);

    node->blurred();
    CHECK(focusable.blurredCount == 1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 4 awaits typed Rust execution"]
fn wave_b_focus_test_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusNode without Focusable doesn't crash", "[FocusNode]")
{
    auto node = make_rcp<FocusNode>();

    // These should not crash
    CHECK(node->keyInput(Key::a, KeyModifiers::none, true, false) == false);
    CHECK(node->textInput("hello") == false);
    node->focused();
    node->blurred();
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 5 awaits typed Rust execution"]
fn wave_b_focus_test_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusNode setFocusable/clearFocusable", "[FocusNode]")
{
    MockFocusable focusable;
    auto node = make_rcp<FocusNode>();

    CHECK(node->focusable() == nullptr);

    node->setFocusable(&focusable);
    CHECK(node->focusable() == &focusable);

    node->clearFocusable();
    CHECK(node->focusable() == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 6 awaits typed Rust execution"]
fn wave_b_focus_test_006_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusNode hierarchy", "[FocusNode]")
{
    auto parent = make_rcp<FocusNode>();
    auto child1 = make_rcp<FocusNode>();
    auto child2 = make_rcp<FocusNode>();

    parent->addChild(child1);
    parent->addChild(child2);

    CHECK(child1->parent() == parent.get());
    CHECK(child2->parent() == parent.get());
    CHECK(parent->children().size() == 2);
    CHECK(parent->isScope() == true);

    parent->removeChild(child1);
    CHECK(child1->parent() == nullptr);
    CHECK(parent->children().size() == 1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 7 awaits typed Rust execution"]
fn wave_b_focus_test_007_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager basic focus operations", "[FocusManager]")
{
    FocusManager manager;
    MockFocusable focusable;
    auto node = make_rcp<FocusNode>(&focusable);

    CHECK(manager.primaryFocus() == nullptr);

    manager.addChild(nullptr, node);
    manager.setFocus(node);

    CHECK(manager.primaryFocus() == node);
    CHECK(manager.hasFocus(node) == true);
    CHECK(manager.hasPrimaryFocus(node) == true);
    CHECK(focusable.focusedCount == 1);

    manager.clearFocus();
    CHECK(manager.primaryFocus() == nullptr);
    CHECK(focusable.blurredCount == 1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 8 awaits typed Rust execution"]
fn wave_b_focus_test_008_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager focus change notifications", "[FocusManager]")
{
    FocusManager manager;
    MockFocusable focusable1, focusable2;
    auto node1 = make_rcp<FocusNode>(&focusable1);
    auto node2 = make_rcp<FocusNode>(&focusable2);

    manager.addChild(nullptr, node1);
    manager.addChild(nullptr, node2);

    manager.setFocus(node1);
    CHECK(focusable1.focusedCount == 1);
    CHECK(focusable1.blurredCount == 0);

    manager.setFocus(node2);
    CHECK(focusable1.blurredCount == 1);
    CHECK(focusable2.focusedCount == 1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 9 awaits typed Rust execution"]
fn wave_b_focus_test_009_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager respects canFocus", "[FocusManager]")
{
    FocusManager manager;
    auto node = make_rcp<FocusNode>();
    node->canFocus(false);

    manager.addChild(nullptr, node);
    manager.setFocus(node);

    CHECK(manager.primaryFocus() == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 10 awaits typed Rust execution"]
fn wave_b_focus_test_010_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager hierarchy", "[FocusManager]")
{
    FocusManager manager;
    auto parent = make_rcp<FocusNode>();
    auto child1 = make_rcp<FocusNode>();
    auto child2 = make_rcp<FocusNode>();

    manager.addChild(nullptr, parent);
    manager.addChild(parent, child1);
    manager.addChild(parent, child2);

    CHECK(parent->parent() == nullptr);
    CHECK(child1->parent() == parent.get());
    CHECK(child2->parent() == parent.get());

    CHECK(parent->isScope() == true);
    CHECK(child1->isScope() == false);

    const auto& children = parent->children();
    CHECK(children.size() == 2);

    // Manager reference is set on all nodes
    CHECK(parent->manager() == &manager);
    CHECK(child1->manager() == &manager);
    CHECK(child2->manager() == &manager);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 11 awaits typed Rust execution"]
fn wave_b_focus_test_011_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager hasFocus with descendants", "[FocusManager]")
{
    FocusManager manager;
    auto parent = make_rcp<FocusNode>();
    auto child = make_rcp<FocusNode>();

    manager.addChild(nullptr, parent);
    manager.addChild(parent, child);

    manager.setFocus(child);

    // Manager queries should work
    CHECK(manager.hasFocus(parent) == true);
    CHECK(manager.hasPrimaryFocus(parent) == false);
    CHECK(manager.hasFocus(child) == true);
    CHECK(manager.hasPrimaryFocus(child) == true);

    // Node's hasFocus flag should be set for focused node and ancestors
    CHECK(parent->hasFocus() == true);
    CHECK(child->hasFocus() == true);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 12 awaits typed Rust execution"]
fn wave_b_focus_test_012_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager removeChild clears focus", "[FocusManager]")
{
    FocusManager manager;
    MockFocusable focusable;
    auto node = make_rcp<FocusNode>(&focusable);

    manager.addChild(nullptr, node);
    manager.setFocus(node);
    CHECK(manager.primaryFocus() == node);

    manager.removeChild(node);
    CHECK(manager.primaryFocus() == nullptr);
    CHECK(focusable.blurredCount == 1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 13 awaits typed Rust execution"]
fn wave_b_focus_test_013_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE(
    "List row reparent: FocusNode removeFromParent preserves primary focus",
    "[FocusManager][list]")
{
    FocusManager manager;
    MockFocusable fLeaf;
    auto scope = make_rcp<FocusNode>(nullptr);
    scope->canFocus(true);
    scope->canTraverse(true);
    auto row = make_rcp<FocusNode>(nullptr);
    row->canFocus(true);
    row->canTraverse(true);
    auto leaf = make_rcp<FocusNode>(&fLeaf);

    manager.addChild(nullptr, scope);
    manager.addChild(scope, row);
    manager.addChild(row, leaf);
    manager.setFocus(leaf);
    CHECK(manager.primaryFocus() == leaf);

    row->removeFromParent();
    CHECK(manager.primaryFocus() == leaf);

    manager.addChild(scope, row, 0);
    CHECK(manager.primaryFocus() == leaf);
    CHECK(fLeaf.blurredCount == 0);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 14 awaits typed Rust execution"]
fn wave_b_focus_test_014_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("hasFocusableContent invalidates when canFocus toggles after caching",
          "[FocusManager]")
{
    FocusManager manager;
    // Both structural: no focusable backing, canFocus=false.
    auto scope = FocusNode::makeStructuralScope();
    auto child = FocusNode::makeStructuralScope();
    manager.addChild(nullptr, scope);
    manager.addChild(scope, child);

    // Compute + cache the "no focusable content" answer.
    CHECK(manager.hasFocusableContent() == false);

    // A canFocus flip on a cached tree must be reflected.
    child->canFocus(true);
    CHECK(manager.hasFocusableContent() == true);

    child->canFocus(false);
    CHECK(manager.hasFocusableContent() == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 15 awaits typed Rust execution"]
fn wave_b_focus_test_015_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE(
    "hasFocusableContent invalidates when focusable backing toggles after "
    "caching",
    "[FocusManager]")
{
    FocusManager manager;
    MockFocusable focusable;
    auto scope = FocusNode::makeStructuralScope();
    auto child = FocusNode::makeStructuralScope();
    manager.addChild(nullptr, scope);
    manager.addChild(scope, child);

    CHECK(manager.hasFocusableContent() == false);

    // Gaining a focusable backing counts even while canFocus stays false.
    child->setFocusable(&focusable);
    CHECK(manager.hasFocusableContent() == true);

    child->clearFocusable();
    CHECK(manager.hasFocusableContent() == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 16 awaits typed Rust execution"]
fn wave_b_focus_test_016_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("hasFocusableContent invalidates when a backed node is added then "
          "removed",
          "[FocusManager]")
{
    // Mirrors a data-bound nested-artboard swap: a structural scope gains a
    // focusable node on swap-in, then loses it on swap-out.
    FocusManager manager;
    MockFocusable focusable;
    auto scope = FocusNode::makeStructuralScope();
    manager.addChild(nullptr, scope);

    CHECK(manager.hasFocusableContent() == false);

    auto backed = make_rcp<FocusNode>(&focusable);
    manager.addChild(scope, backed);
    CHECK(manager.hasFocusableContent() == true);

    manager.removeChild(backed);
    CHECK(manager.hasFocusableContent() == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 17 awaits typed Rust execution"]
fn wave_b_focus_test_017_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("hasFocusableContent invalidates when the last root is erased",
          "[FocusManager]")
{
    // eraseRoot is the only invalidation for a root removed while migrating to
    // another manager; exercise it directly via a re-parent to a second
    // manager, which erases the node from the first manager's root list.
    FocusManager first;
    FocusManager second;
    auto node = make_rcp<FocusNode>();
    node->canFocus(true);
    first.addChild(nullptr, node);

    CHECK(first.hasFocusableContent() == true);

    // Migrating the root out of `first` empties its tree.
    second.addChild(nullptr, node);
    CHECK(first.hasFocusableContent() == false);
    CHECK(second.hasFocusableContent() == true);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 18 awaits typed Rust execution"]
fn wave_b_focus_test_018_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager input routing", "[FocusManager]")
{
    FocusManager manager;
    MockFocusable focusable;
    focusable.returnValue = true;
    auto node = make_rcp<FocusNode>(&focusable);

    manager.addChild(nullptr, node);

    // No focus, input not handled
    CHECK(manager.keyInput(Key::a, KeyModifiers::none, true, false) == false);
    CHECK(manager.textInput("hello") == false);
    GamepadSnapshot snap{};
    snap.deviceId = 1;
    snap.buttonMask = 1;
    CHECK(manager.gamepadDispatch(ListenerInvocation::gamepadConnected(snap)) ==
          false);

    manager.setFocus(node);

    // With focus, input is routed
    CHECK(manager.keyInput(Key::b, KeyModifiers::none, true, false) == true);
    CHECK(focusable.keyInputCount == 1);
    CHECK(focusable.lastKey == Key::b);

    CHECK(manager.textInput("world") == true);
    CHECK(focusable.textInputCount == 1);
    CHECK(focusable.lastText == "world");

    CHECK(manager.gamepadDispatch(ListenerInvocation::gamepadConnected(snap)) ==
          true);
    CHECK(focusable.gamepadDispatchCount == 1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 19 awaits typed Rust execution"]
fn wave_b_focus_test_019_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager traversal basic", "[FocusManager]")
{
    FocusManager manager;
    MockFocusable f1, f2, f3;
    auto node1 = make_rcp<FocusNode>(&f1);
    auto node2 = make_rcp<FocusNode>(&f2);
    auto node3 = make_rcp<FocusNode>(&f3);

    manager.addChild(nullptr, node1);
    manager.addChild(nullptr, node2);
    manager.addChild(nullptr, node3);

    // Focus first node
    manager.setFocus(node1);
    CHECK(manager.primaryFocus() == node1);

    // Navigate forward
    manager.focusNext();
    CHECK(manager.primaryFocus() == node2);

    manager.focusNext();
    CHECK(manager.primaryFocus() == node3);

    // Navigate backward
    manager.focusPrevious();
    CHECK(manager.primaryFocus() == node2);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 20 awaits typed Rust execution"]
fn wave_b_focus_test_020_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager traversal with tabIndex", "[FocusManager]")
{
    FocusManager manager;
    auto node1 = make_rcp<FocusNode>();
    auto node2 = make_rcp<FocusNode>();
    auto node3 = make_rcp<FocusNode>();

    node1->tabIndex(3);
    node2->tabIndex(1);
    node3->tabIndex(2);

    manager.addChild(nullptr, node1);
    manager.addChild(nullptr, node2);
    manager.addChild(nullptr, node3);

    // Start with no focus, focusNext should pick first by tabIndex
    manager.focusNext();
    CHECK(manager.primaryFocus() == node2); // tabIndex 1

    manager.focusNext();
    CHECK(manager.primaryFocus() == node3); // tabIndex 2

    manager.focusNext();
    CHECK(manager.primaryFocus() == node1); // tabIndex 3
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 21 awaits typed Rust execution"]
fn wave_b_focus_test_021_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager traversal skips non-traversable", "[FocusManager]")
{
    FocusManager manager;
    auto node1 = make_rcp<FocusNode>();
    auto node2 = make_rcp<FocusNode>();
    auto node3 = make_rcp<FocusNode>();

    node2->canTraverse(false);

    manager.addChild(nullptr, node1);
    manager.addChild(nullptr, node2);
    manager.addChild(nullptr, node3);

    manager.setFocus(node1);
    manager.focusNext();

    // Should skip node2 and go to node3
    CHECK(manager.primaryFocus() == node3);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 22 awaits typed Rust execution"]
fn wave_b_focus_test_022_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager edge behavior closedLoop", "[FocusManager]")
{
    FocusManager manager;
    auto scope = make_rcp<FocusNode>();
    auto node1 = make_rcp<FocusNode>();
    auto node2 = make_rcp<FocusNode>();

    scope->edgeBehavior(EdgeBehavior::closedLoop);

    manager.addChild(nullptr, scope);
    manager.addChild(scope, node1);
    manager.addChild(scope, node2);

    manager.setFocus(node2);
    manager.focusNext();

    // Should wrap to first
    CHECK(manager.primaryFocus() == node1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 23 awaits typed Rust execution"]
fn wave_b_focus_test_023_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager edge behavior stop", "[FocusManager]")
{
    FocusManager manager;
    auto scope = make_rcp<FocusNode>();
    auto node1 = make_rcp<FocusNode>();
    auto node2 = make_rcp<FocusNode>();

    scope->edgeBehavior(EdgeBehavior::stop);

    manager.addChild(nullptr, scope);
    manager.addChild(scope, node1);
    manager.addChild(scope, node2);

    manager.setFocus(node2);
    manager.focusNext();

    // Should stay on node2
    CHECK(manager.primaryFocus() == node2);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 24 awaits typed Rust execution"]
fn wave_b_focus_test_024_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager ancestor notification on focus", "[FocusManager]")
{
    FocusManager manager;
    MockFocusable grandparentFocusable, parentFocusable, childFocusable;
    auto grandparent = make_rcp<FocusNode>(&grandparentFocusable);
    auto parent = make_rcp<FocusNode>(&parentFocusable);
    auto child = make_rcp<FocusNode>(&childFocusable);

    manager.addChild(nullptr, grandparent);
    manager.addChild(grandparent, parent);
    manager.addChild(parent, child);

    // Focus the leaf node
    manager.setFocus(child);

    // All ancestors should have received focused() callback
    CHECK(childFocusable.focusedCount == 1);
    CHECK(parentFocusable.focusedCount == 1);
    CHECK(grandparentFocusable.focusedCount == 1);

    // All nodes in the chain should have hasFocus flag
    CHECK(child->hasFocus() == true);
    CHECK(parent->hasFocus() == true);
    CHECK(grandparent->hasFocus() == true);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 25 awaits typed Rust execution"]
fn wave_b_focus_test_025_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager common ancestor optimization", "[FocusManager]")
{
    FocusManager manager;
    MockFocusable parentFocusable, child1Focusable, child2Focusable;
    auto parent = make_rcp<FocusNode>(&parentFocusable);
    auto child1 = make_rcp<FocusNode>(&child1Focusable);
    auto child2 = make_rcp<FocusNode>(&child2Focusable);

    manager.addChild(nullptr, parent);
    manager.addChild(parent, child1);
    manager.addChild(parent, child2);

    // Focus first child
    manager.setFocus(child1);
    CHECK(parentFocusable.focusedCount == 1);
    CHECK(child1Focusable.focusedCount == 1);

    // Move focus to sibling - parent should NOT get re-notified
    manager.setFocus(child2);
    CHECK(child1Focusable.blurredCount == 1);
    CHECK(child2Focusable.focusedCount == 1);
    // Parent should not be blurred or re-focused
    CHECK(parentFocusable.focusedCount == 1); // Still 1, not 2
    CHECK(parentFocusable.blurredCount == 0);

    // Parent still has focus (descendant focused)
    CHECK(parent->hasFocus() == true);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 26 awaits typed Rust execution"]
fn wave_b_focus_test_026_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager traversal focuses leaves only", "[FocusManager]")
{
    FocusManager manager;
    MockFocusable scopeFocusable, leaf1Focusable, leaf2Focusable;
    auto scope = make_rcp<FocusNode>(&scopeFocusable);
    auto leaf1 = make_rcp<FocusNode>(&leaf1Focusable);
    auto leaf2 = make_rcp<FocusNode>(&leaf2Focusable);

    manager.addChild(nullptr, scope);
    manager.addChild(scope, leaf1);
    manager.addChild(scope, leaf2);

    // Start with no focus, focusNext should focus first leaf, not scope
    manager.focusNext();
    CHECK(manager.primaryFocus() == leaf1);
    CHECK(manager.hasPrimaryFocus(scope) == false);
    CHECK(scope->hasFocus() == true); // But scope has descendant focus

    manager.focusNext();
    CHECK(manager.primaryFocus() == leaf2);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 27 awaits typed Rust execution"]
fn wave_b_focus_test_027_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager nested scopes focus deepest leaf", "[FocusManager]")
{
    FocusManager manager;
    auto scope1 = make_rcp<FocusNode>();
    auto scope2 = make_rcp<FocusNode>();
    auto leaf = make_rcp<FocusNode>();

    manager.addChild(nullptr, scope1);
    manager.addChild(scope1, scope2);
    manager.addChild(scope2, leaf);

    // Navigate should go directly to the deepest leaf
    manager.focusNext();
    CHECK(manager.primaryFocus() == leaf);
    CHECK(scope1->hasFocus() == true);
    CHECK(scope2->hasFocus() == true);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 28 awaits typed Rust execution"]
fn wave_b_focus_test_028_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager edge behavior parentScope exits to parent",
          "[FocusManager]")
{
    FocusManager manager;
    auto root = make_rcp<FocusNode>();
    auto scope = make_rcp<FocusNode>();
    auto inner1 = make_rcp<FocusNode>();
    auto inner2 = make_rcp<FocusNode>();
    auto outer = make_rcp<FocusNode>();

    scope->edgeBehavior(EdgeBehavior::parentScope);

    manager.addChild(nullptr, root);
    manager.addChild(root, scope);
    manager.addChild(scope, inner1);
    manager.addChild(scope, inner2);
    manager.addChild(root, outer);

    // Focus last node in scope
    manager.setFocus(inner2);
    CHECK(manager.primaryFocus() == inner2);

    // Navigate forward should exit scope and go to outer
    manager.focusNext();
    CHECK(manager.primaryFocus() == outer);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 29 awaits typed Rust execution"]
fn wave_b_focus_test_029_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager clearFocus clears hasFocus flag chain",
          "[FocusManager]")
{
    FocusManager manager;
    MockFocusable parentFocusable, childFocusable;
    auto parent = make_rcp<FocusNode>(&parentFocusable);
    auto child = make_rcp<FocusNode>(&childFocusable);

    manager.addChild(nullptr, parent);
    manager.addChild(parent, child);

    manager.setFocus(child);
    CHECK(parent->hasFocus() == true);
    CHECK(child->hasFocus() == true);

    manager.clearFocus();

    // Both should be cleared
    CHECK(parent->hasFocus() == false);
    CHECK(child->hasFocus() == false);

    // Both should have received blurred callback
    CHECK(parentFocusable.blurredCount == 1);
    CHECK(childFocusable.blurredCount == 1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 30 awaits typed Rust execution"]
fn wave_b_focus_test_030_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager removeChild clears manager reference", "[FocusManager]")
{
    FocusManager manager;
    auto node = make_rcp<FocusNode>();

    manager.addChild(nullptr, node);
    CHECK(node->manager() == &manager);

    manager.removeChild(node);
    CHECK(node->manager() == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 31 awaits typed Rust execution"]
fn wave_b_focus_test_031_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Freeing a FocusNode clears the parent pointer of a child that "
          "outlives it",
          "[FocusManager]")
{
    FocusManager manager;
    auto scope = make_rcp<FocusNode>(); // persistent host scope, held here
    {
        auto row = make_rcp<FocusNode>(); // transient list row
        manager.addChild(nullptr, row);
        manager.addChild(row, scope);
        CHECK(scope->parent() == row.get());
        // The list re-sync removes the row from the manager, then drops it.
        manager.removeChild(row);
    } // row FocusNode destroyed here; scope survives via the outer rcp

    REQUIRE(scope->parent() == nullptr);

    // Re-homing the survivor is now safe — no dereference of the freed row.
    auto newParent = make_rcp<FocusNode>();
    manager.addChild(nullptr, newParent);
    manager.addChild(newParent, scope);
    CHECK(scope->parent() == newParent.get());
    CHECK(newParent->children().size() == 1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 32 awaits typed Rust execution"]
fn wave_b_focus_test_032_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager::addChild removes a migrating root from its previous "
          "manager",
          "[FocusManager]")
{
    FocusManager internalManager;
    FocusManager parentManager;
    auto scope = make_rcp<FocusNode>();

    internalManager.addChild(nullptr, scope);
    CHECK(scope->manager() == &internalManager);
    CHECK(internalManager.rootNodes().size() == 1);

    // Migrate the scope to the parent manager (no FocusNode parent -> root).
    parentManager.addChild(nullptr, scope);
    CHECK(scope->manager() == &parentManager);
    CHECK(parentManager.rootNodes().size() == 1);

    // The internal manager must no longer reference the migrated scope.
    CHECK(internalManager.rootNodes().empty());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 33 awaits typed Rust execution"]
fn wave_b_focus_test_033_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("A migrated focus scope survives destruction of its previous manager",
          "[FocusManager]")
{
    FocusManager parentManager;
    auto scope = make_rcp<FocusNode>();
    {
        FocusManager internalManager;
        internalManager.addChild(nullptr, scope);
        parentManager.addChild(nullptr, scope); // migrate to parent
        CHECK(scope->manager() == &parentManager);
    } // internalManager destroyed here

    // The scope still belongs to parentManager, not the destroyed one.
    CHECK(scope->manager() == &parentManager);

    if (scope->manager() != nullptr)
    {
        scope->manager()->removeChild(scope);
    }
    CHECK(parentManager.rootNodes().empty());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 34 awaits typed Rust execution"]
fn wave_b_focus_test_034_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager traversal backward from first leaf exits scope",
          "[FocusManager]")
{
    FocusManager manager;
    auto root = make_rcp<FocusNode>();
    auto before = make_rcp<FocusNode>();
    auto scope = make_rcp<FocusNode>();
    auto inner = make_rcp<FocusNode>();

    scope->edgeBehavior(EdgeBehavior::parentScope);

    manager.addChild(nullptr, root);
    manager.addChild(root, before);
    manager.addChild(root, scope);
    manager.addChild(scope, inner);

    // Focus the inner node
    manager.setFocus(inner);

    // Navigate backward should exit scope and go to before
    manager.focusPrevious();
    CHECK(manager.primaryFocus() == before);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 35 awaits typed Rust execution"]
fn wave_b_focus_test_035_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager closedLoop wraps backward", "[FocusManager]")
{
    FocusManager manager;
    auto scope = make_rcp<FocusNode>();
    auto node1 = make_rcp<FocusNode>();
    auto node2 = make_rcp<FocusNode>();

    scope->edgeBehavior(EdgeBehavior::closedLoop);

    manager.addChild(nullptr, scope);
    manager.addChild(scope, node1);
    manager.addChild(scope, node2);

    manager.setFocus(node1);
    manager.focusPrevious();

    // Should wrap to last
    CHECK(manager.primaryFocus() == node2);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 36 awaits typed Rust execution"]
fn wave_b_focus_test_036_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager stop prevents backward traversal", "[FocusManager]")
{
    FocusManager manager;
    auto scope = make_rcp<FocusNode>();
    auto node1 = make_rcp<FocusNode>();
    auto node2 = make_rcp<FocusNode>();

    scope->edgeBehavior(EdgeBehavior::stop);

    manager.addChild(nullptr, scope);
    manager.addChild(scope, node1);
    manager.addChild(scope, node2);

    manager.setFocus(node1);
    manager.focusPrevious();

    // Should stay on node1
    CHECK(manager.primaryFocus() == node1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 37 awaits typed Rust execution"]
fn wave_b_focus_test_037_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("StateMachineInstance hasFocusNodes ignores non-traversable scopes",
          "[FocusManager]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    auto scope = make_rcp<FocusNode>();
    scope->canFocus(false);
    scope->canTraverse(false);
    smi.focusManager()->addChild(nullptr, scope);
    CHECK(smi.hasFocusNodes() == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 38 awaits typed Rust execution"]
fn wave_b_focus_test_038_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("StateMachineInstance hasFocusNodes sees leaves under a "
          "transparent scope",
          "[FocusManager]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    // Transparent structural scope as registered for a data-bound nested
    // artboard host: unbacked (no focusable), canFocus/canTraverse/canTouch
    // false. Traversal descends through it because it has no focusable.
    auto scope = make_rcp<FocusNode>();
    scope->canFocus(false);
    scope->canTraverse(false);
    scope->canTouch(false);
    smi.focusManager()->addChild(nullptr, scope);

    // Empty scope contributes no focus targets (e.g. a bindable artboard with
    // no focus nodes).
    CHECK(smi.hasFocusNodes() == false);

    // Swapping in an artboard that has a focusable leaf must make the state
    // machine report focus nodes, even though the leaf lives under the scope.
    auto leaf = make_rcp<FocusNode>();
    smi.focusManager()->addChild(scope, leaf);
    CHECK(smi.hasFocusNodes() == true);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 39 awaits typed Rust execution"]
fn wave_b_focus_test_039_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("StateMachineInstance hasFocusNodes counts focus data that is "
          "currently ineligible for traversal",
          "[FocusManager]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    // hasFocusNodes gates one-time setup in high-level runtimes (attaching
    // tab/shift+tab listeners in JS), so authored focus data must count even
    // while it can't currently be focused: canFocus/canTraverse are
    // data-bindable and collapse/visibility can change on any frame.
    FocusData focusData;
    // canFocus/canTraverse are now bits in the focusFlags bitmask; clear both
    // (leave the rest) to make the node ineligible for traversal.
    focusData.focusFlags(
        focusData.focusFlags() &
        ~(FocusData::canFocusBitmask | FocusData::canTraverseBitmask));
    smi.focusManager()->addChild(nullptr, focusData.focusNode());
    CHECK(smi.hasFocusNodes() == true);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 40 awaits typed Rust execution"]
fn wave_b_focus_test_040_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager traversal descends through a transparent scope "
          "and keeps sibling order",
          "[FocusManager]")
{
    FocusManager manager;
    auto leafA = make_rcp<FocusNode>();
    auto scope = make_rcp<FocusNode>();
    auto leafC = make_rcp<FocusNode>();

    // scope mirrors a data-bound nested artboard host slot sitting between two
    // sibling focus nodes: unbacked (no focusable) and not a focus target
    // itself, but Tab descends through it to whatever artboard is swapped in.
    scope->canFocus(false);
    scope->canTraverse(false);
    scope->canTouch(false);

    manager.addChild(nullptr, leafA);
    manager.addChild(nullptr, scope);
    manager.addChild(nullptr, leafC);

    // Empty scope is skipped: A -> C.
    manager.focusNext();
    CHECK(manager.primaryFocus() == leafA);
    manager.focusNext();
    CHECK(manager.primaryFocus() == leafC);

    // Populate the scope (artboard swapped in). Its leaf occupies the scope's
    // sibling slot, so traversal order becomes A -> B -> C.
    manager.clearFocus();
    auto leafB = make_rcp<FocusNode>();
    manager.addChild(scope, leafB);

    manager.focusNext();
    CHECK(manager.primaryFocus() == leafA);
    manager.focusNext();
    CHECK(manager.primaryFocus() == leafB);
    manager.focusNext();
    CHECK(manager.primaryFocus() == leafC);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 41 awaits typed Rust execution"]
fn wave_b_focus_test_041_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager drops focus when a leaf under a transparent scope "
          "becomes hidden",
          "[FocusManager]")
{
    FocusManager manager;
    // Unbacked scope: the shape of a data-bound nested artboard's scope node.
    auto scope = make_rcp<FocusNode>();
    scope->canFocus(false);
    scope->canTraverse(false);
    scope->canTouch(false);
    // A focusable leaf inside it, like a swapped-in nested artboard's element.
    MockFocusable leafFocusable;
    auto leaf = make_rcp<FocusNode>(&leafFocusable);
    manager.addChild(nullptr, scope);
    manager.addChild(scope, leaf);

    // Tab descends through the scope onto the nested leaf.
    manager.focusNext();
    REQUIRE(manager.primaryFocus() == leaf);

    // Hide the nested content (its focusable reports ineligible). Focus must be
    // dropped, not left stranded behind the scope.
    leafFocusable.eligible = false;
    manager.dropFocusIfFocusTargetHidden();
    CHECK(manager.primaryFocus() == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 42 awaits typed Rust execution"]
fn wave_b_focus_test_042_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager rebuilding one scope's subtree preserves focus in a "
          "sibling scope",
          "[FocusManager]")
{
    FocusManager manager;
    // Two sibling transparent scopes, like two data-bound nested artboard
    // hosts.
    auto scopeA = make_rcp<FocusNode>();
    scopeA->canFocus(false);
    scopeA->canTraverse(false);
    scopeA->canTouch(false);
    auto scopeB = make_rcp<FocusNode>();
    scopeB->canFocus(false);
    scopeB->canTraverse(false);
    scopeB->canTouch(false);

    MockFocusable leafAFocusable, leafBFocusable;
    auto leafA = make_rcp<FocusNode>(&leafAFocusable);
    auto leafB = make_rcp<FocusNode>(&leafBFocusable);
    manager.addChild(nullptr, scopeA);
    manager.addChild(scopeA, leafA);
    manager.addChild(nullptr, scopeB);
    manager.addChild(scopeB, leafB);

    // Focus the leaf inside scope A.
    manager.setFocus(leafA);
    REQUIRE(manager.primaryFocus() == leafA);

    // Simulate swapping the artboard in sibling scope B: tear down B's current
    // content and rebuild it with a new focusable leaf under the same scope.
    // Focus held in the unrelated scope A must be untouched.
    manager.removeChild(leafB);
    MockFocusable leafB2Focusable;
    auto leafB2 = make_rcp<FocusNode>(&leafB2Focusable);
    manager.addChild(scopeB, leafB2);

    CHECK(manager.primaryFocus() == leafA);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 43 awaits typed Rust execution"]
fn wave_b_focus_test_043_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusActionTraversal perform advances focus with traversalKind next",
          "[FocusActionTraversal]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    FocusManager* fm = smi.focusManager();
    MockFocusable f1, f2;
    auto node1 = make_rcp<FocusNode>(&f1);
    auto node2 = make_rcp<FocusNode>(&f2);
    fm->addChild(nullptr, node1);
    fm->addChild(nullptr, node2);
    fm->setFocus(node1);

    FocusActionTraversal action;
    action.traversalKind(0);
    action.perform(&smi, ListenerInvocation::none());

    CHECK(fm->primaryFocus() == node2);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 44 awaits typed Rust execution"]
fn wave_b_focus_test_044_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusActionTraversal perform moves focus back with traversalKind "
          "previous",
          "[FocusActionTraversal]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    FocusManager* fm = smi.focusManager();
    MockFocusable f1, f2;
    auto node1 = make_rcp<FocusNode>(&f1);
    auto node2 = make_rcp<FocusNode>(&f2);
    fm->addChild(nullptr, node1);
    fm->addChild(nullptr, node2);
    fm->setFocus(node2);

    FocusActionTraversal action;
    action.traversalKind(1);
    action.perform(&smi, ListenerInvocation::none());

    CHECK(fm->primaryFocus() == node1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 45 awaits typed Rust execution"]
fn wave_b_focus_test_045_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusActionTraversal perform unknown traversalKind defaults to next",
          "[FocusActionTraversal]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    FocusManager* fm = smi.focusManager();
    MockFocusable f1, f2;
    auto node1 = make_rcp<FocusNode>(&f1);
    auto node2 = make_rcp<FocusNode>(&f2);
    fm->addChild(nullptr, node1);
    fm->addChild(nullptr, node2);
    fm->setFocus(node1);

    FocusActionTraversal action;
    action.traversalKind(999);
    action.perform(&smi, ListenerInvocation::none());

    CHECK(fm->primaryFocus() == node2);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 46 awaits typed Rust execution"]
fn wave_b_focus_test_046_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE(
    "StateMachineInstance exposes hasFocusNodes, focusNext, focusPrevious from focusManager",
    "[FocusManager]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    MockFocusable f1, f2;
    auto node1 = make_rcp<FocusNode>(&f1);
    auto node2 = make_rcp<FocusNode>(&f2);

    CHECK(smi.hasFocusNodes() == false);

    smi.focusManager()->addChild(nullptr, node1);
    smi.focusManager()->addChild(nullptr, node2);
    smi.focusManager()->setFocus(node1);

    CHECK(smi.hasFocusNodes() == true);
    CHECK(smi.focusNext() == true);
    CHECK(smi.focusPrevious() == true);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 47 awaits typed Rust execution"]
fn wave_b_focus_test_047_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusActionTraversal perform ignores null StateMachineInstance",
          "[FocusActionTraversal]")
{
    FocusActionTraversal action;
    action.traversalKind(0);
    action.perform(nullptr, ListenerInvocation::none());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 48 awaits typed Rust execution"]
fn wave_b_focus_test_048_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Focusable::acceptsKeyboardInput defaults to false", "[Focusable]")
{
    MockFocusable f;
    CHECK(f.acceptsKeyboardInput() == false);

    KeyboardAcceptingFocusable kf;
    CHECK(kf.acceptsKeyboardInput() == true);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 49 awaits typed Rust execution"]
fn wave_b_focus_test_049_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("StateMachineInstance::focusState reports no focus when nothing is "
          "focused",
          "[FocusState]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    auto state = smi.focusState();
    CHECK(state.hasFocus == false);
    CHECK(state.expectsKeyboardInput == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 50 awaits typed Rust execution"]
fn wave_b_focus_test_050_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("StateMachineInstance::focusState reports focused non-keyboard "
          "focusable",
          "[FocusState]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    MockFocusable f;
    auto node = make_rcp<FocusNode>(&f);
    smi.focusManager()->addChild(nullptr, node);
    smi.focusManager()->setFocus(node);

    auto state = smi.focusState();
    CHECK(state.hasFocus == true);
    CHECK(state.expectsKeyboardInput == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 51 awaits typed Rust execution"]
fn wave_b_focus_test_051_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("StateMachineInstance::focusState reports keyboard expectation when "
          "focused focusable accepts keys",
          "[FocusState]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    KeyboardAcceptingFocusable kf;
    auto node = make_rcp<FocusNode>(&kf);
    smi.focusManager()->addChild(nullptr, node);
    smi.focusManager()->setFocus(node);

    auto state = smi.focusState();
    CHECK(state.hasFocus == true);
    CHECK(state.expectsKeyboardInput == true);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 52 awaits typed Rust execution"]
fn wave_b_focus_test_052_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("StateMachineInstance::focusState clears when focus is cleared",
          "[FocusState]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    KeyboardAcceptingFocusable kf;
    auto node = make_rcp<FocusNode>(&kf);
    smi.focusManager()->addChild(nullptr, node);
    smi.focusManager()->setFocus(node);

    REQUIRE(smi.focusState().hasFocus == true);

    smi.focusManager()->clearFocus();

    auto state = smi.focusState();
    CHECK(state.hasFocus == false);
    CHECK(state.expectsKeyboardInput == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 53 awaits typed Rust execution"]
fn wave_b_focus_test_053_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("StateMachineInstance::focusState tracks switches between focusables",
          "[FocusState]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    MockFocusable plain;
    KeyboardAcceptingFocusable kf;
    auto plainNode = make_rcp<FocusNode>(&plain);
    auto kfNode = make_rcp<FocusNode>(&kf);
    smi.focusManager()->addChild(nullptr, plainNode);
    smi.focusManager()->addChild(nullptr, kfNode);

    smi.focusManager()->setFocus(plainNode);
    {
        auto state = smi.focusState();
        CHECK(state.hasFocus == true);
        CHECK(state.expectsKeyboardInput == false);
    }

    smi.focusManager()->setFocus(kfNode);
    {
        auto state = smi.focusState();
        CHECK(state.hasFocus == true);
        CHECK(state.expectsKeyboardInput == true);
    }

    smi.focusManager()->setFocus(plainNode);
    {
        auto state = smi.focusState();
        CHECK(state.hasFocus == true);
        CHECK(state.expectsKeyboardInput == false);
    }
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 54 awaits typed Rust execution"]
fn wave_b_focus_test_054_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("StateMachineInstance::focusState uses external focus manager when "
          "set",
          "[FocusState]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    FocusManager external;
    KeyboardAcceptingFocusable kf;
    auto node = make_rcp<FocusNode>(&kf);
    external.addChild(nullptr, node);
    external.setFocus(node);

    // Before swapping, internal manager has nothing focused.
    CHECK(smi.focusState().hasFocus == false);

    smi.setExternalFocusManager(&external);

    auto state = smi.focusState();
    CHECK(state.hasFocus == true);
    CHECK(state.expectsKeyboardInput == true);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 55 awaits typed Rust execution"]
fn wave_b_focus_test_055_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("StateMachineInstance::clearFocus clears internal focus manager",
          "[FocusState]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    KeyboardAcceptingFocusable kf;
    auto node = make_rcp<FocusNode>(&kf);
    smi.focusManager()->addChild(nullptr, node);
    smi.focusManager()->setFocus(node);

    REQUIRE(smi.focusState().hasFocus == true);

    smi.clearFocus();

    auto state = smi.focusState();
    CHECK(state.hasFocus == false);
    CHECK(state.expectsKeyboardInput == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 56 awaits typed Rust execution"]
fn wave_b_focus_test_056_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager setFocus on a scope descends to first leaf",
          "[FocusManager]")
{
    FocusManager manager;
    auto scope = make_rcp<FocusNode>();
    auto leaf1 = make_rcp<FocusNode>();
    auto leaf2 = make_rcp<FocusNode>();

    manager.addChild(nullptr, scope);
    manager.addChild(scope, leaf1);
    manager.addChild(scope, leaf2);

    // Focusing the scope resolves to its first eligible leaf.
    manager.setFocus(scope);
    CHECK(manager.primaryFocus() == leaf1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 57 awaits typed Rust execution"]
fn wave_b_focus_test_057_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager setFocus on a scope descends depth-first",
          "[FocusManager]")
{
    FocusManager manager;
    auto scope = make_rcp<FocusNode>();
    auto row = make_rcp<FocusNode>();
    auto leaf = make_rcp<FocusNode>();
    auto sibling = make_rcp<FocusNode>();

    manager.addChild(nullptr, scope);
    manager.addChild(scope, row);
    manager.addChild(row, leaf);
    manager.addChild(scope, sibling);

    // Depth-first: first leaf is the leaf nested under the first child (row).
    manager.setFocus(scope);
    CHECK(manager.primaryFocus() == leaf);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 58 awaits typed Rust execution"]
fn wave_b_focus_test_058_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager setFocus on a scope with no eligible leaf falls back",
          "[FocusManager]")
{
    FocusManager manager;
    auto scope = make_rcp<FocusNode>();
    auto child = make_rcp<FocusNode>();
    // Child cannot be traversed, so the scope has no eligible leaf to descend
    // to. The scope itself remains the focus target (preserves prior behavior).
    child->canTraverse(false);

    manager.addChild(nullptr, scope);
    manager.addChild(scope, child);

    manager.setFocus(scope);
    CHECK(manager.primaryFocus() == scope);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 59 awaits typed Rust execution"]
fn wave_b_focus_test_059_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager setFocus on an ineligible scope is a no-op",
          "[FocusManager]")
{
    FocusManager manager;
    auto scope = make_rcp<FocusNode>();
    auto leaf = make_rcp<FocusNode>();
    // The requested target itself cannot be focused. Descent must not reach an
    // eligible descendant — focus stays unchanged (no-op), matching the prior
    // early-return guard behavior.
    scope->canFocus(false);

    manager.addChild(nullptr, scope);
    manager.addChild(scope, leaf);

    manager.setFocus(scope);
    CHECK(manager.primaryFocus() == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 60 awaits typed Rust execution"]
fn wave_b_focus_test_060_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager setFocus on a leaf is unchanged", "[FocusManager]")
{
    FocusManager manager;
    auto scope = make_rcp<FocusNode>();
    auto leaf1 = make_rcp<FocusNode>();
    auto leaf2 = make_rcp<FocusNode>();

    manager.addChild(nullptr, scope);
    manager.addChild(scope, leaf1);
    manager.addChild(scope, leaf2);

    // Directly focusing a leaf still focuses that exact leaf (no-op descent).
    manager.setFocus(leaf2);
    CHECK(manager.primaryFocus() == leaf2);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 61 awaits typed Rust execution"]
fn wave_b_focus_test_061_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager Tab after focusing a scope traverses leaf siblings",
          "[FocusManager]")
{
    FocusManager manager;
    auto scope = make_rcp<FocusNode>();
    auto leaf1 = make_rcp<FocusNode>();
    auto leaf2 = make_rcp<FocusNode>();

    manager.addChild(nullptr, scope);
    manager.addChild(scope, leaf1);
    manager.addChild(scope, leaf2);

    // Focusing the scope lands on the first leaf; Tab then advances to the
    // scope's next leaf rather than skipping the scope's children.
    manager.setFocus(scope);
    CHECK(manager.primaryFocus() == leaf1);

    manager.focusNext();
    CHECK(manager.primaryFocus() == leaf2);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 62 awaits typed Rust execution"]
fn wave_b_focus_test_062_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusActionClear perform clears the primary focus",
          "[FocusActionClear]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    FocusManager* fm = smi.focusManager();
    MockFocusable f1;
    auto node1 = make_rcp<FocusNode>(&f1);
    fm->addChild(nullptr, node1);
    fm->setFocus(node1);
    REQUIRE(fm->primaryFocus() == node1);

    FocusActionClear action;
    action.perform(&smi, ListenerInvocation::none());

    CHECK(fm->primaryFocus() == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 63 awaits typed Rust execution"]
fn wave_b_focus_test_063_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusActionClear perform is a no-op when nothing is focused",
          "[FocusActionClear]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    REQUIRE(smi.focusManager()->primaryFocus() == nullptr);

    FocusActionClear action;
    action.perform(&smi, ListenerInvocation::none());

    CHECK(smi.focusManager()->primaryFocus() == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 64 awaits typed Rust execution"]
fn wave_b_focus_test_064_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusActionClear perform ignores null StateMachineInstance",
          "[FocusActionClear]")
{
    FocusActionClear action;
    // Must not dereference the null instance.
    action.perform(nullptr, ListenerInvocation::none());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 65 awaits typed Rust execution"]
fn wave_b_focus_test_065_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("TransitionFocusCondition uses the reassigned core type key",
          "[TransitionFocusCondition]")
{
    // Locks in the collision fix: master's font PR claimed 1035, so this
    // condition was reassigned to 1038. A regression here means a type-key
    // clash on import/export.
    // Copy into a local to avoid ODR-using the in-class static constant
    // (which has no out-of-line definition) when binding it to Catch2's
    // by-reference comparison expressions.
    uint16_t typeKey = TransitionFocusConditionBase::typeKey;
    CHECK(typeKey == 1038);

    auto condition = std::make_unique<TransitionFocusCondition>();
    CHECK(condition->coreType() == typeKey);
    CHECK(condition->is<TransitionFocusCondition>());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 66 awaits typed Rust execution"]
fn wave_b_focus_test_066_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("TransitionFocusCondition evaluate returns false for a null "
          "StateMachineInstance",
          "[TransitionFocusCondition]")
{
    // Heap allocation value-initializes the (comparator) members to null, so
    // the guard clauses and destructor are well-defined even without import.
    auto condition = std::make_unique<TransitionFocusCondition>();
    CHECK(condition->evaluate(nullptr, nullptr) == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 67 awaits typed Rust execution"]
fn wave_b_focus_test_067_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("TransitionFocusCondition evaluate returns false when no target "
          "comparator is configured",
          "[TransitionFocusCondition]")
{
    NoOpFactory factory;
    Artboard artboard(&factory);
    auto instance = artboard.instance();
    StateMachine machine;
    StateMachineInstance smi(&machine, instance.get());

    auto condition = std::make_unique<TransitionFocusCondition>();
    // With neither comparator set to a TransitionPropertyComponentComparator,
    // there is no focus target to evaluate against, so the condition is false.
    CHECK(condition->evaluate(&smi, nullptr) == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 68 awaits typed Rust execution"]
fn wave_b_focus_test_068_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Swapping bindable artboard registers nested focus nodes for Tab",
          "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/bindable_focus_tree_swap.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(vmi != nullptr);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);

    auto* focusManager = stateMachine->focusManager();
    REQUIRE(focusManager != nullptr);
    REQUIRE(stateMachine->hasFocusNodes() == true);

    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    REQUIRE(focusManager->primaryFocus() != nullptr);

    CHECK(stateMachine->focusNext() == false);
    // There's only one focus node in the main artboard, go back to that last
    // node
    stateMachine->focusPrevious();

    auto* artboardProp = vmi->propertyValue("bindedArt");
    REQUIRE(artboardProp != nullptr);
    REQUIRE(artboardProp->is<rive::ViewModelInstanceArtboard>());
    auto* vmiArtboard = artboardProp->as<rive::ViewModelInstanceArtboard>();

    // Has other focus nodes in this artboard
    auto focusableSource = file->bindableArtboardNamed("Focusable");
    REQUIRE(focusableSource != nullptr);

    vmiArtboard->asset(focusableSource);
    stateMachine->advanceAndApply(0.016f);

    rive::NestedArtboard* focusableHost = nullptr;
    for (auto* nestedHost : artboard->nestedArtboards())
    {
        auto* source = nestedHost->sourceArtboard();
        if (source != nullptr && source->name() == "Focusable")
        {
            focusableHost = nestedHost;
            break;
        }
    }
    REQUIRE(focusableHost != nullptr);
    auto* focusableInstance = focusableHost->artboardInstance(0);
    REQUIRE(focusableInstance != nullptr);

    CHECK(stateMachine->focusNext() == true);
    CHECK(focusManager->primaryFocus() != nullptr);
    CHECK(focusManager->primaryFocusImmediateArtboard() == focusableInstance);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 69 awaits typed Rust execution"]
fn wave_b_focus_test_069_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Swapping a bindable nested artboard preserves focus held elsewhere",
          "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/bindable_focus_tree_swap.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(vmi != nullptr);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);

    auto* focusManager = stateMachine->focusManager();
    REQUIRE(focusManager != nullptr);

    // Focus the main artboard's own focus node. Before the swap the bindable
    // host is "Plain" (no focus nodes), so the main node is the only focusable.
    focusManager->focusNext();
    auto focused = focusManager->primaryFocus();
    REQUIRE(focused != nullptr);
    REQUIRE(focusManager->primaryFocusImmediateArtboard() == artboard.get());

    // Swap the (unrelated) bindable nested artboard to one that HAS focus
    // nodes.
    auto* artboardProp = vmi->propertyValue("bindedArt");
    REQUIRE(artboardProp != nullptr);
    REQUIRE(artboardProp->is<rive::ViewModelInstanceArtboard>());
    auto* vmiArtboard = artboardProp->as<rive::ViewModelInstanceArtboard>();
    auto focusableSource = file->bindableArtboardNamed("Focusable");
    REQUIRE(focusableSource != nullptr);
    vmiArtboard->asset(focusableSource);
    stateMachine->advanceAndApply(0.016f);

    // Focus held on the main artboard must survive the unrelated nested swap:
    // the swap only re-syncs the swapped host's subtree, not the whole tree.
    CHECK(focusManager->primaryFocus() == focused);
    CHECK(focusManager->primaryFocusImmediateArtboard() == artboard.get());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 70 awaits typed Rust execution"]
fn wave_b_focus_test_070_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("FocusManager skips collapsed nodes and fully transparent nodes",
          "[FocusManager]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/focus_collapsing.riv", &silver);

    auto artboard = file->artboardDefault();
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    auto focusManager = artboard->focusManager();
    auto opacityProp =
        vmi->propertyValue("opacity")->as<rive::ViewModelInstanceNumber>();
    auto isMainLayout2VisibleProp = vmi->propertyValue("isMainLayout2Visible")
                                        ->as<rive::ViewModelInstanceBoolean>();

    stateMachine->bindViewModelInstance(vmi);
    // ===> Frame 0
    auto renderer = silver.makeRenderer();
    stateMachine->advanceAndApply(0.016f);
    // ===> Frame 1
    artboard->draw(renderer.get());
    silver.addFrame();

    focusManager->focusNext();
    // The first focusable is now inside a data-bound nested artboard
    REQUIRE(focusManager->primaryFocus() != nullptr);
    REQUIRE(focusManager->primaryFocusImmediateArtboard() != nullptr);
    REQUIRE(focusManager->primaryFocusImmediateArtboard() != artboard.get());
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    // ===> Frame 2
    silver.addFrame();

    // Tab next into the main artboard's own element — the one `opacity`
    // controls.
    focusManager->focusNext();
    REQUIRE(focusManager->primaryFocus() != nullptr);
    REQUIRE(focusManager->primaryFocusImmediateArtboard() == artboard.get());
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Hide that focused element; focus must be dropped.
    opacityProp->propertyValue(0);
    // First advance sets the opacity to 0
    stateMachine->advanceAndApply(0.016f);
    // Next frame the focus is dropped
    stateMachine->advanceAndApply(0.016f);
    REQUIRE(focusManager->primaryFocus() == nullptr);
    artboard->draw(renderer.get());
    // ===> Frame 3
    silver.addFrame();

    opacityProp->propertyValue(1);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    // ===> Frame 4
    silver.addFrame();
    focusManager->focusNext();
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    REQUIRE(focusManager->primaryFocus() != nullptr);
    artboard->draw(renderer.get());
    // ===> Frame 5
    silver.addFrame();
    isMainLayout2VisibleProp->propertyValue(false);
    stateMachine->advanceAndApply(0.016f);
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    // ===> Frame 6
    silver.addFrame();

    // Toggles only between visible focused elements
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    // ===> Frame 7
    silver.addFrame();
    focusManager->focusNext();
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    // ===> Frame 8
    silver.addFrame();

    // Fully rotates over all nodes
    isMainLayout2VisibleProp->propertyValue(true);
    stateMachine->advanceAndApply(0.016f);
    focusManager->focusNext();
    artboard->draw(renderer.get());
    // ===> Frame 9
    silver.addFrame();
    stateMachine->advanceAndApply(0.016f);
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    // ===> Frame 10
    silver.addFrame();
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    focusManager->focusNext();
    artboard->draw(renderer.get());
    // ===> Frame 11
    silver.addFrame();
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    // ===> Frame 12
    silver.addFrame();
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("focus_collapsing"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 71 awaits typed Rust execution"]
fn wave_b_focus_test_071_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Focused elements receive keyboard inputs", "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/keyboard_listener.riv", &silver);

    auto artboard = file->artboardDefault();
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    auto renderer = silver.makeRenderer();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    auto focusManager = artboard->focusManager();
    // Child index 5
    focusManager->focusPrevious();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    focusManager->keyInput(rive::Key::space,
                           rive::KeyModifiers::none,
                           false,
                           false);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Child index 4
    focusManager->focusPrevious();
    // Child index 3
    focusManager->focusPrevious();
    // Child index 2
    focusManager->focusPrevious();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    focusManager->keyInput(rive::Key::space,
                           rive::KeyModifiers::none,
                           false,
                           false);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Child index 1
    focusManager->focusPrevious();
    // Child index 0
    focusManager->focusPrevious();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    focusManager->keyInput(rive::Key::space,
                           rive::KeyModifiers::none,
                           false,
                           false);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    focusManager->focusPrevious();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    focusManager->keyInput(rive::Key::space,
                           rive::KeyModifiers::none,
                           false,
                           false);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("keyboard_listener"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 72 awaits typed Rust execution"]
fn wave_b_focus_test_072_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Keyboard inputs with different key combinations", "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/keyboard_listener.riv", &silver);

    auto artboard = file->artboardNamed("KeyboardInput");
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);
    auto keyCountProp =
        vmi->propertyValue("keyCount")->as<rive::ViewModelInstanceNumber>();

    stateMachine->bindViewModelInstance(vmi);
    auto renderer = silver.makeRenderer();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    auto focusManager = artboard->focusManager();
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    // Key "a" on phase down with no modifiers is captured
    focusManager->keyInput(rive::Key::a, rive::KeyModifiers::none, true, false);
    stateMachine->advanceAndApply(0.016f);
    CHECK(keyCountProp->propertyValue() == 1);
    artboard->draw(renderer.get());
    silver.addFrame();
    // Key "a" on phase repeat with no modifiers is not captured
    focusManager->keyInput(rive::Key::a, rive::KeyModifiers::none, true, true);
    stateMachine->advanceAndApply(0.016f);
    CHECK(keyCountProp->propertyValue() == 1);
    // Key "a" on phase up with no modifiers is captured
    focusManager->keyInput(rive::Key::a,
                           rive::KeyModifiers::none,
                           false,
                           false);
    stateMachine->advanceAndApply(0.016f);
    CHECK(keyCountProp->propertyValue() == 2);

    // Key "a" on phase down with modifiers is not captured
    focusManager->keyInput(rive::Key::a,
                           rive::KeyModifiers::shift,
                           true,
                           false);
    stateMachine->advanceAndApply(0.016f);
    CHECK(keyCountProp->propertyValue() == 2);

    // Key "e" on any phase is not captured
    focusManager->keyInput(rive::Key::e,
                           rive::KeyModifiers::none,
                           false,
                           false);
    focusManager->keyInput(rive::Key::e, rive::KeyModifiers::none, true, true);
    focusManager->keyInput(rive::Key::e, rive::KeyModifiers::none, true, false);
    CHECK(keyCountProp->propertyValue() == 2);
    stateMachine->advanceAndApply(0.016f);
    // Key "b" on phase down with no modifiers is NOT captured
    focusManager->keyInput(rive::Key::b, rive::KeyModifiers::none, true, false);
    // Key "b" on phase up with no modifiers is NOT captured
    CHECK(keyCountProp->propertyValue() == 2);
    stateMachine->advanceAndApply(0.016f);
    focusManager->keyInput(rive::Key::b,
                           rive::KeyModifiers::none,
                           false,
                           false);
    stateMachine->advanceAndApply(0.016f);
    CHECK(keyCountProp->propertyValue() == 3);
    // Key "b" on phase repeat with no modifiers is captured
    focusManager->keyInput(rive::Key::b, rive::KeyModifiers::none, true, true);
    stateMachine->advanceAndApply(0.016f);
    CHECK(keyCountProp->propertyValue() == 4);
    // Key "d" on phase down with no modifiers is not captured
    focusManager->keyInput(rive::Key::d, rive::KeyModifiers::none, true, false);
    stateMachine->advanceAndApply(0.016f);
    CHECK(keyCountProp->propertyValue() == 4);
    // Key "d" on phase down with shift + command modifiers is captured
    focusManager->keyInput(rive::Key::d,
                           rive::KeyModifiers::shift | rive::KeyModifiers::meta,
                           true,
                           false);
    stateMachine->advanceAndApply(0.016f);
    CHECK(keyCountProp->propertyValue() == 5);
    // Key "c" on phase down with shift + command modifiers is NOT captured
    focusManager->keyInput(rive::Key::c,
                           rive::KeyModifiers::shift | rive::KeyModifiers::meta,
                           true,
                           false);
    stateMachine->advanceAndApply(0.016f);
    CHECK(keyCountProp->propertyValue() == 5);
    // Key "c" on phase down with shift modifiers is captured
    focusManager->keyInput(rive::Key::c,
                           rive::KeyModifiers::shift,
                           true,
                           false);
    stateMachine->advanceAndApply(0.016f);
    CHECK(keyCountProp->propertyValue() == 6);
    // Key "x" on phase down with shift modifiers is NOT captured
    focusManager->keyInput(rive::Key::x,
                           rive::KeyModifiers::shift,
                           true,
                           false);
    stateMachine->advanceAndApply(0.016f);
    CHECK(keyCountProp->propertyValue() == 6);

    artboard->draw(renderer.get());

    CHECK(silver.matches("keyboard_listener-KeyboardInput"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 73 awaits typed Rust execution"]
fn wave_b_focus_test_073_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Text input events are handled on focused nodes", "[silver]")
{
    auto file = ReadRiveFile("assets/text_input_event.riv");

    auto artboard = file->artboardDefault();

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi = file->createViewModelInstance(artboard.get());
    auto isFocusedProp =
        vmi->propertyValue("isFocused")->as<rive::ViewModelInstanceBoolean>();
    auto hasKeyedProp =
        vmi->propertyValue("hasKeyed")->as<rive::ViewModelInstanceBoolean>();
    auto hasTextedProp =
        vmi->propertyValue("hasTexted")->as<rive::ViewModelInstanceBoolean>();

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);

    auto focusManager = artboard->focusManager();
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    CHECK(isFocusedProp->propertyValue() == true);
    CHECK(hasKeyedProp->propertyValue() == false);
    CHECK(hasTextedProp->propertyValue() == false);

    // Key "b" on phase down with no modifiers is NOT captured
    focusManager->keyInput(rive::Key::b, rive::KeyModifiers::none, true, false);
    stateMachine->advanceAndApply(0.016f);
    CHECK(isFocusedProp->propertyValue() == true);
    CHECK(hasKeyedProp->propertyValue() == false);
    CHECK(hasTextedProp->propertyValue() == false);
    // Text "b" on captured by text but not by key
    focusManager->textInput("b");
    stateMachine->advanceAndApply(0.016f);
    CHECK(isFocusedProp->propertyValue() == true);
    CHECK(hasKeyedProp->propertyValue() == false);
    CHECK(hasTextedProp->propertyValue() == true);

    // Key "a" on phase down with no modifiers is captured by key
    focusManager->keyInput(rive::Key::a, rive::KeyModifiers::none, true, false);
    stateMachine->advanceAndApply(0.016f);
    CHECK(isFocusedProp->propertyValue() == true);
    CHECK(hasKeyedProp->propertyValue() == true);
    CHECK(hasTextedProp->propertyValue() == true);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 74 awaits typed Rust execution"]
fn wave_b_focus_test_074_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Focus traversal listener actions", "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/focus_traversal.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());

    auto renderer = silver.makeRenderer();
    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // There are 2 rows of buttons
    // Top row: Top / Right / Down / Left
    // Bottom row: Prev / Next

    // Click on Next
    stateMachine->pointerDown(rive::Vec2D(180, 450));
    stateMachine->pointerUp(rive::Vec2D(180, 450));
    stateMachine->advanceAndApply(0.016f);
    // Second advance to apply focus changes
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Click on Prev twice to reenter focus tree
    stateMachine->pointerDown(rive::Vec2D(60, 450));
    stateMachine->pointerUp(rive::Vec2D(60, 450));
    stateMachine->advanceAndApply(0.016f);
    stateMachine->pointerDown(rive::Vec2D(60, 450));
    stateMachine->pointerUp(rive::Vec2D(60, 450));
    stateMachine->advanceAndApply(0.016f);
    // Second advance to apply focus changes
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Click on Up
    stateMachine->pointerDown(rive::Vec2D(60, 350));
    stateMachine->pointerUp(rive::Vec2D(60, 350));
    stateMachine->advanceAndApply(0.016f);
    // Second advance to apply focus changes
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Click on Left
    stateMachine->pointerDown(rive::Vec2D(420, 350));
    stateMachine->pointerUp(rive::Vec2D(420, 350));
    stateMachine->advanceAndApply(0.016f);
    // Second advance to apply focus changes
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Click on Down
    stateMachine->pointerDown(rive::Vec2D(300, 350));
    stateMachine->pointerUp(rive::Vec2D(300, 350));
    stateMachine->advanceAndApply(0.016f);
    // Second advance to apply focus changes
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Click on Right
    stateMachine->pointerDown(rive::Vec2D(180, 350));
    stateMachine->pointerUp(rive::Vec2D(180, 350));
    stateMachine->advanceAndApply(0.016f);
    // Second advance to apply focus changes
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("focus_traversal"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 75 awaits typed Rust execution"]
fn wave_b_focus_test_075_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Focus traversal clears focus when it reaches edge of root scope",
          "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/focusable_element.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi = file->createViewModelInstance(artboard.get());

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);
    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->focusManager()->focusNext();
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->focusManager()->focusNext();
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->focusManager()->focusNext();
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->focusManager()->focusNext();
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->focusManager()->focusNext();
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->focusManager()->focusNext();
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->focusManager()->focusNext();
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("focusable_element"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 76 awaits typed Rust execution"]
fn wave_b_focus_test_076_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ArtboardComponentList list scope is registered on shared "
          "FocusManager",
          "[FocusManager][list]")
{
    auto file = ReadRiveFile("assets/component_list_1.riv");
    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(vmi != nullptr);
    artboard->bindViewModelInstance(vmi);
    auto sm = artboard->stateMachineAt(0);
    REQUIRE(sm != nullptr);
    artboard->advance(0.0f);

    auto* list = artboard->find<rive::ArtboardComponentList>("List");
    REQUIRE(list != nullptr);
    auto* fm = artboard->focusManager();
    REQUIRE(fm != nullptr);

    artboard->buildFocusTree(artboard->focusManager(), nullptr);
    auto scope = list->listScopeFocusNode();
    REQUIRE(scope != nullptr);
    CHECK(scope->manager() == fm);
    CHECK(scope->name() == "ArtboardComponentListScope");
    // Transparent structural scope: not a focus target itself; traversal
    // descends through it (focusNodeTraversable) to reach item focusables.
    CHECK(scope->canFocus() == false);
    CHECK(scope->canTraverse() == false);
    CHECK(scope->focusable() == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 77 awaits typed Rust execution"]
fn wave_b_focus_test_077_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("List under Node: when parent has a direct FocusData, "
          "findClosestFocusNode from list matches that node",
          "[FocusManager][list]")
{
    // buildFocusTreeVisit pass-1: at most one direct child FocusData per
    // container; if present, its focusNode is the scope for siblings (e.g. the
    // list host). The walk-based fallback from the old findClosest for the
    // no-direct-FocusData case is not used by the focus build anymore.
    auto file = ReadRiveFile("assets/component_list_1.riv");
    auto artboard = file->artboard("Main")->instance();
    REQUIRE(artboard != nullptr);
    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(vmi != nullptr);
    artboard->bindViewModelInstance(vmi);
    auto sm = artboard->stateMachineAt(0);
    REQUIRE(sm != nullptr);
    artboard->advance(0.0f);

    auto* list = artboard->find<rive::ArtboardComponentList>("List");
    REQUIRE(list != nullptr);
    auto* p = list->parent();
    REQUIRE(p != nullptr);
    REQUIRE(p->is<rive::Node>());

    rive::rcp<rive::FocusNode> fromFirstDirectFd;
    for (auto* ch : p->as<rive::Node>()->children())
    {
        if (ch != nullptr && ch->is<rive::FocusData>())
        {
            fromFirstDirectFd = ch->as<rive::FocusData>()->focusNode();
            break;
        }
    }
    if (fromFirstDirectFd != nullptr)
    {
        CHECK(rive::FocusData::findClosestFocusNode(list) == fromFirstDirectFd);
    }
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 78 awaits typed Rust execution"]
fn wave_b_focus_test_078_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Focus is correctly built and updated for lists", "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/list_focus_order.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    auto focusManager = stateMachine->focusManager();

    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    auto stageProcessedProp = vmi->propertyValue("stageProcessed")
                                  ->as<rive::ViewModelInstanceBoolean>();
    auto stageCountProp =
        vmi->propertyValue("stageCount")->as<rive::ViewModelInstanceNumber>();

    auto renderer = silver.makeRenderer();
    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Focuses on first element of tree
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Focuses on last element of list
    focusManager->focusNext();
    focusManager->focusNext();
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Inserts one element at end of list
    stageProcessedProp->propertyValue(false);
    stageCountProp->propertyValue(1);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Focus is on that new element
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Focused elements is moved in the list and keeps focus
    stageProcessedProp->propertyValue(false);
    stageCountProp->propertyValue(2);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Focusing on the next element correctly focuses on the next element on the
    // list
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Removing the focused element from the list, clears the focus
    stageProcessedProp->propertyValue(false);
    stageCountProp->propertyValue(3);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    // Focuses back on first element of tree
    focusManager->focusNext();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("list_focus_order"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 79 awaits typed Rust execution"]
fn wave_b_focus_test_079_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Focus based transitions work", "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/focus_test.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());

    auto renderer = silver.makeRenderer();
    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    stateMachine->pointerDown(rive::Vec2D(55.0, 65.0));
    stateMachine->pointerUp(rive::Vec2D(55.0, 65.0));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();

    stateMachine->pointerDown(rive::Vec2D(442.0, 65.0));
    stateMachine->pointerUp(rive::Vec2D(442.0, 65.0));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("focus_test"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 80 awaits typed Rust execution"]
fn wave_b_focus_test_080_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("List item focus tree stays under its row when the item's state "
          "machine is (re)wired during the focus sync",
          "[FocusManager][list]")
{
    // Regression for the syncListRowNodesWithList ordering bug: each list
    // item's state machine must be wired to the shared FocusManager BEFORE the
    // item's focus tree is (re)built under its row. setExternalFocusManager
    // rebuilds the item's focus tree at the manager ROOT as a side effect, so
    // if it runs after the build-under-row it clobbers the row placement and
    // the item's focus nodes end up detached from the list scope (at the
    // manager root).
    //
    // The natural build path happens to wire the manager first (via
    // linkStateMachineToArtboard, whose setExternalFocusManager runs before the
    // row sync), so the in-loop call is normally skipped by the
    // `smi->focusManager() != fm` guard. Force the mismatch to exercise the
    // ordering directly.
    auto file = ReadRiveFile("assets/list_focus_order.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);
    auto* fm = stateMachine->focusManager();
    REQUIRE(fm != nullptr);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(vmi != nullptr);
    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);

    REQUIRE(artboard->artboardComponentLists().size() == 1);
    auto* list = artboard->artboardComponentLists()[0];
    REQUIRE(list != nullptr);
    const int itemCount = static_cast<int>(list->artboardCount());
    REQUIRE(itemCount > 0);

    // A row node with children means the item's focus subtree is parented under
    // it (inside the list scope) — the invariant the bug breaks.
    auto rowForItem = [&](int i) -> rive::FocusNode* {
        auto scope = list->listScopeFocusNode();
        if (scope == nullptr || i >= static_cast<int>(scope->children().size()))
        {
            return nullptr;
        }
        return scope->children()[static_cast<size_t>(i)].get();
    };

    // Pick a list item that (after the normal build) has focus content placed
    // under its row AND owns a state machine — the only case where the in-loop
    // setExternalFocusManager fires.
    int targetIndex = -1;
    for (int i = 0; i < itemCount; i++)
    {
        rive::FocusNode* row = rowForItem(i);
        if (row != nullptr && !row->children().empty() &&
            list->stateMachineInstance(i) != nullptr)
        {
            targetIndex = i;
            break;
        }
    }
    REQUIRE(targetIndex != -1);

    // Force the mismatch: drop the item's shared-manager wiring so the next
    // focus sync must call setExternalFocusManager(fm) again — the exact call
    // whose manager-root rebuild would clobber the row placement if it ran
    // after the build-under-row.
    list->stateMachineInstance(targetIndex)->setExternalFocusManager(nullptr);
    CHECK(list->stateMachineInstance(targetIndex)->focusManager() != fm);

    // Re-run the parent focus build; this recreates the list scope/rows and
    // re-syncs each item under its row.
    artboard->cleanupFocusTree();
    artboard->buildFocusTree(fm, nullptr);

    // With the fix (wire first, place last) the item's focus subtree is
    // parented under its row inside the list scope. With the bug it was rebuilt
    // at the manager root, leaving the row empty.
    rive::FocusNode* targetRow = rowForItem(targetIndex);
    REQUIRE(targetRow != nullptr);
    CHECK(targetRow->manager() == fm);
    CHECK_FALSE(targetRow->children().empty());
    CHECK(list->stateMachineInstance(targetIndex)->focusManager() == fm);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 81 awaits typed Rust execution"]
fn wave_b_focus_test_081_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Swappable artboard slot keeps its place in tab order",
          "[FocusManager]")
{
    // File: https://editor.uat.rive.app/file/untitled/36028
    auto file = ReadRiveFile("assets/swappable_artboards_focus.riv");
    auto artboard = file->artboardNamed("Main");
    REQUIRE(artboard != nullptr);

    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);
    auto* focusManager = stateMachine->focusManager();
    REQUIRE(focusManager != nullptr);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(vmi != nullptr);
    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);
    stateMachine->advanceAndApply(0.016f);

    // Only the data-bound slot is flagged as swappable; static nested
    // artboards get no placeholder scope regardless of whether their artboard
    // contains focusables.
    rive::NestedArtboard* slotHost = nullptr;
    for (auto* host : artboard->nestedArtboards())
    {
        auto* source = host->sourceArtboard();
        REQUIRE(source != nullptr);
        if (source->name() == "Swappable1" || source->name() == "Swappable2")
        {
            CHECK(host->isArtboardDataBound() == true);
            slotHost = host;
        }
        else
        {
            CHECK(host->isArtboardDataBound() == false);
        }
    }
    REQUIRE(slotHost != nullptr);

    CHECK(stateMachine->hasFocusNodes() == true);

    // The artboard owning the currently focused element.
    auto focusedArtboardName = [&]() -> std::string {
        auto* ab = focusManager->primaryFocusImmediateArtboard();
        return ab != nullptr ? ab->name() : "<none>";
    };

    // Initial tab order follows the Main hierarchy: Rectangle (Main) -> slot
    // (Swappable1) -> StaticNestWithFocusable. StaticNestWithoutFocusable
    // contributes nothing.
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "Main");
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "Swappable1");
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "StaticNestWithFocusable");
    // Edge of the root scope clears focus.
    CHECK(stateMachine->focusNext() == false);
    CHECK(focusManager->primaryFocus() == nullptr);

    // Swap the slot to an artboard with no focusables: the slot contributes
    // no focus stop and the rest of the order is untouched.
    auto* artboardProp = vmi->propertyValue("artboardProp");
    REQUIRE(artboardProp != nullptr);
    REQUIRE(artboardProp->is<rive::ViewModelInstanceArtboard>());
    auto* vmiArtboard = artboardProp->as<rive::ViewModelInstanceArtboard>();
    auto swappable2 = file->bindableArtboardNamed("Swappable2");
    REQUIRE(swappable2 != nullptr);
    vmiArtboard->asset(swappable2);
    stateMachine->advanceAndApply(0.016f);

    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "Main");
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "StaticNestWithFocusable");
    CHECK(stateMachine->focusNext() == false);

    // Focus the Main rectangle, then swap back to the focusable artboard:
    // focus held elsewhere survives the (unrelated) swap...
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "Main");
    auto heldFocus = focusManager->primaryFocus();
    auto swappable1 = file->bindableArtboardNamed("Swappable1");
    REQUIRE(swappable1 != nullptr);
    vmiArtboard->asset(swappable1);
    stateMachine->advanceAndApply(0.016f);
    CHECK(focusManager->primaryFocus() == heldFocus);
    CHECK(focusedArtboardName() == "Main");

    // ...and the swapped-in focusable takes the slot's place in the middle of
    // the tab order (its hierarchy position), not the end.
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "Swappable1");
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "StaticNestWithFocusable");
    CHECK(stateMachine->focusNext() == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 82 awaits typed Rust execution"]
fn wave_b_focus_test_082_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Repeat focus-tree build keeps focus inside an untouched nested "
          "artboard",
          "[FocusManager]")
{
    // #4 regression: a second full buildFocusTree pass over an already-wired
    // tree (same manager) must not tear down and rebuild nested artboards that
    // did not change — doing so blurs focus resting inside them. Only the
    // non-destructive scope placement should run on the repeat pass.
    auto file = ReadRiveFile("assets/swappable_artboards_focus.riv");
    auto artboard = file->artboardNamed("Main");
    REQUIRE(artboard != nullptr);

    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);
    auto* focusManager = stateMachine->focusManager();
    REQUIRE(focusManager != nullptr);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(vmi != nullptr);
    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);
    stateMachine->advanceAndApply(0.016f);

    auto focusedArtboardName = [&]() -> std::string {
        auto* ab = focusManager->primaryFocusImmediateArtboard();
        return ab != nullptr ? ab->name() : "<none>";
    };

    // Tab into the focusable that lives inside the STATIC nested artboard.
    // Order (established by the sibling test): Main -> Swappable1 ->
    // StaticNestWithFocusable.
    CHECK(stateMachine->focusNext() == true);
    CHECK(stateMachine->focusNext() == true);
    CHECK(stateMachine->focusNext() == true);
    REQUIRE(focusedArtboardName() == "StaticNestWithFocusable");
    auto heldFocus = focusManager->primaryFocus();
    REQUIRE(heldFocus != nullptr);

    // Repeat the full build pass with the SAME manager (mirrors the host's
    // documented two-phase build, or any later focus-tree re-wire). Nothing
    // about the static nested artboard changed, so the focus resting inside it
    // must survive rather than being blurred by a needless rebuild.
    artboard->buildFocusTree(focusManager, nullptr);

    CHECK(focusManager->primaryFocus() == heldFocus);
    CHECK(focusedArtboardName() == "StaticNestWithFocusable");
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 83 awaits typed Rust execution"]
fn wave_b_focus_test_083_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Cross-file swaps keep slot order and share the focus manager",
          "[FocusManager]")
{
    // The slot's host, bind, and scope all live in the main file; the
    // swapped-in artboard may come from a different .riv. Loading the asset
    // twice yields two independent Files, so pulling bindable artboards from
    // the second File exercises the cross-file path.
    auto file = ReadRiveFile("assets/swappable_artboards_focus.riv");
    auto otherFile = ReadRiveFile("assets/swappable_artboards_focus.riv");

    auto artboard = file->artboardNamed("Main");
    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);
    auto* focusManager = stateMachine->focusManager();
    REQUIRE(focusManager != nullptr);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(vmi != nullptr);
    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);

    auto focusedArtboard = [&]() -> rive::Artboard* {
        return focusManager->primaryFocusImmediateArtboard();
    };
    auto focusedArtboardName = [&]() -> std::string {
        auto* ab = focusedArtboard();
        return ab != nullptr ? ab->name() : "<none>";
    };
    // The slot host's bound state machine (created by the latest swap).
    auto slotBoundStateMachine = [&]() -> rive::StateMachineInstance* {
        for (auto* host : artboard->nestedArtboards())
        {
            if (!host->isArtboardDataBound())
            {
                continue;
            }
            for (auto* animation : host->nestedAnimations())
            {
                if (animation->is<rive::NestedStateMachine>())
                {
                    return animation->as<rive::NestedStateMachine>()
                        ->stateMachineInstance();
                }
            }
        }
        return nullptr;
    };

    auto* artboardProp = vmi->propertyValue("artboardProp");
    REQUIRE(artboardProp != nullptr);
    REQUIRE(artboardProp->is<rive::ViewModelInstanceArtboard>());
    auto* vmiArtboard = artboardProp->as<rive::ViewModelInstanceArtboard>();

    // Swap in a LEAF artboard (one focusable, no nested hosts) from the
    // other file.
    auto foreignSwappable = otherFile->bindableArtboardNamed("Swappable1");
    REQUIRE(foreignSwappable != nullptr);
    vmiArtboard->asset(foreignSwappable);
    stateMachine->advanceAndApply(0.016f);

    // The foreign artboard's focus node sits at the slot's hierarchy
    // position, exactly like a same-file swap.
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "Main");
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "Swappable1");
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "StaticNestWithFocusable");
    CHECK(stateMachine->focusNext() == false);

    // The swapped-in artboard's own state machine must share the parent
    // FocusManager, so its focus/keyboard listener groups act on the same
    // focus state that Tab traversal uses.
    auto* leafSmi = slotBoundStateMachine();
    REQUIRE(leafSmi != nullptr);
    CHECK(leafSmi->focusManager() == focusManager);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 84 awaits typed Rust execution"]
fn wave_b_focus_test_084_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Unresolvable artboard swap leaves focus and tab order untouched",
          "[FocusManager]")
{
    auto file = ReadRiveFile("assets/swappable_artboards_focus.riv");
    auto artboard = file->artboardNamed("Main");
    REQUIRE(artboard != nullptr);

    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);
    auto* focusManager = stateMachine->focusManager();
    REQUIRE(focusManager != nullptr);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(vmi != nullptr);
    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);
    stateMachine->advanceAndApply(0.016f);

    auto focusedArtboardName = [&]() -> std::string {
        auto* ab = focusManager->primaryFocusImmediateArtboard();
        return ab != nullptr ? ab->name() : "<none>";
    };

    // Default order (per the sibling test): Main -> Swappable1 ->
    // StaticNestWithFocusable. Rest focus on Main's Rectangle and hold the rcp.
    CHECK(stateMachine->focusNext() == true);
    REQUIRE(focusedArtboardName() == "Main");
    auto heldFocus = focusManager->primaryFocus();
    REQUIRE(heldFocus != nullptr);

    // Drive the slot's VM artboard property into the UNRESOLVABLE state: no
    // bindable asset and a bogus (non -1) id that matches no artboard. This is
    // distinct from an explicit clear (asset null AND propertyValue == -1), so
    // updateArtboard must return early and leave the on-screen slot alone.
    auto* artboardProp = vmi->propertyValue("artboardProp");
    REQUIRE(artboardProp != nullptr);
    REQUIRE(artboardProp->is<rive::ViewModelInstanceArtboard>());
    auto* vmiArtboard = artboardProp->as<rive::ViewModelInstanceArtboard>();
    vmiArtboard->propertyValue(9999u);
    REQUIRE(vmiArtboard->asset() == nullptr);
    REQUIRE(vmiArtboard->propertyValue() != static_cast<uint32_t>(-1));
    stateMachine->advanceAndApply(0.016f);

    // Focus held on Main survives the failed swap...
    CHECK(focusManager->primaryFocus() == heldFocus);
    CHECK(focusedArtboardName() == "Main");

    // ...and the outgoing Swappable1 kept its focus nodes, so the full tab
    // order is unchanged: Main -> Swappable1 -> StaticNestWithFocusable.
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "Swappable1");
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "StaticNestWithFocusable");
    CHECK(stateMachine->focusNext() == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned focus_test case 85 awaits typed Rust execution"]
fn wave_b_focus_test_085_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Initially-empty bindable slot keeps its authored tab position on "
          "first swap",
          "[FocusManager]")
{
    auto file = ReadRiveFile("assets/swappable_artboards_focus.riv");
    auto artboard = file->artboardNamed("Main");
    REQUIRE(artboard != nullptr);

    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);
    auto* focusManager = stateMachine->focusManager();
    REQUIRE(focusManager != nullptr);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(vmi != nullptr);

    // Clear the slot to explicit null (asset null, propertyValue -1) BEFORE the
    // first advance, so the slot is empty when the focus tree is first built.
    auto* artboardProp = vmi->propertyValue("artboardProp");
    REQUIRE(artboardProp != nullptr);
    REQUIRE(artboardProp->is<rive::ViewModelInstanceArtboard>());
    auto* vmiArtboard = artboardProp->as<rive::ViewModelInstanceArtboard>();
    vmiArtboard->asset(nullptr);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.016f);
    stateMachine->advanceAndApply(0.016f);

    auto focusedArtboardName = [&]() -> std::string {
        auto* ab = focusManager->primaryFocusImmediateArtboard();
        return ab != nullptr ? ab->name() : "<none>";
    };

    // The empty slot's scope holds its place but offers no focus stop, so the
    // order skips it: Main -> StaticNestWithFocusable.
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "Main");
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "StaticNestWithFocusable");
    CHECK(stateMachine->focusNext() == false);

    // Swap Swappable1 in for the first time: it must build under the scope the
    // empty-slot build pass already placed, entering the MIDDLE of the tab
    // order (Main -> Swappable1 -> StaticNestWithFocusable), not the end.
    auto swappable1 = file->bindableArtboardNamed("Swappable1");
    REQUIRE(swappable1 != nullptr);
    vmiArtboard->asset(swappable1);
    stateMachine->advanceAndApply(0.016f);
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "Main");
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "Swappable1");
    CHECK(stateMachine->focusNext() == true);
    CHECK(focusedArtboardName() == "StaticNestWithFocusable");
    CHECK(stateMachine->focusNext() == false);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned follow_path_constraint_test case 1 awaits typed Rust execution"]
fn wave_b_follow_path_constraint_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("follow path constraint updates world transform", "[file]")
{
    auto file = ReadRiveFile("assets/follow_path.riv");

    auto artboard = file->artboard();

    REQUIRE(artboard->find<rive::TransformComponent>("target") != nullptr);
    auto target = artboard->find<rive::TransformComponent>("target");

    REQUIRE(artboard->find<rive::TransformComponent>("rect") != nullptr);
    auto rectangle = artboard->find<rive::TransformComponent>("rect");

    artboard->advance(0.0f);

    auto targetComponents = target->worldTransform().decompose();
    auto rectComponents = rectangle->worldTransform().decompose();
    REQUIRE(targetComponents.x() == rectComponents.x());
    REQUIRE(targetComponents.y() == rectComponents.y());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned follow_path_constraint_test case 2 awaits typed Rust execution"]
fn wave_b_follow_path_constraint_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("follow path with 0 opacity constraint updates world transform",
          "[file]")
{
    auto file = ReadRiveFile("assets/follow_path_with_0_opacity.riv");

    auto artboard = file->artboard();

    REQUIRE(artboard->find<rive::TransformComponent>("target") != nullptr);
    auto target = artboard->find<rive::TransformComponent>("target");

    REQUIRE(artboard->find<rive::TransformComponent>("rect") != nullptr);
    auto rectangle = artboard->find<rive::TransformComponent>("rect");

    artboard->advance(0.0f);

    auto targetComponents = target->worldTransform().decompose();
    auto rectComponents = rectangle->worldTransform().decompose();
    REQUIRE(targetComponents.x() == rectComponents.x());
    REQUIRE(targetComponents.y() == rectComponents.y());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned follow_path_constraint_test case 3 awaits typed Rust execution"]
fn wave_b_follow_path_constraint_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE(
    "follow path constraint with path at 0 opacity updates world transform",
    "[file]")
{
    auto file = ReadRiveFile("assets/follow_path_path_0_opacity.riv");

    auto artboard = file->artboard();

    REQUIRE(artboard->find<rive::TransformComponent>("target") != nullptr);
    auto target = artboard->find<rive::TransformComponent>("target");

    REQUIRE(artboard->find<rive::TransformComponent>("rect") != nullptr);
    auto rectangle = artboard->find<rive::TransformComponent>("rect");

    artboard->advance(0.0f);

    auto targetComponents = target->worldTransform().decompose();
    auto rectComponents = rectangle->worldTransform().decompose();
    REQUIRE(targetComponents.x() == rectComponents.x());
    REQUIRE(targetComponents.y() == rectComponents.y());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned follow_path_constraint_test case 4 awaits typed Rust execution"]
fn wave_b_follow_path_constraint_test_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Animate shape along follow path", "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/follow_path_shapes.riv", &silver);

    auto artboard = file->artboardDefault();

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = 60;
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("follow_path_animate_shape"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned follow_path_constraint_test case 5 awaits typed Rust execution"]
fn wave_b_follow_path_constraint_test_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Animate solo along follow path", "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/follow_path_solos.riv", &silver);

    auto artboard = file->artboardDefault();

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = 240;
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("follow_path_animate_solo"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned follow_path_constraint_test case 6 awaits typed Rust execution"]
fn wave_b_follow_path_constraint_test_006_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Animate follow path target path", "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/follow_path_path.riv", &silver);

    auto artboard = file->artboardDefault();

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = 120;
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("follow_path_animate_target"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned follow_path_constraint_test case 7 awaits typed Rust execution"]
fn wave_b_follow_path_constraint_test_007_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Text follow path modifier", "[silver]")
{
    rive::SerializingFactory silver;
    auto file =
        ReadRiveFile("assets/text_follow_path_shape_length.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    auto viewModelInstance =
        file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(viewModelInstance != nullptr);
    artboard->bindViewModelInstance(viewModelInstance);

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    int frames = 10;
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("text_follow_path_shape_length"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned follow_path_constraint_test case 8 awaits typed Rust execution"]
fn wave_b_follow_path_constraint_test_008_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Follow path constraint with path as target", "[silver]")
{
    rive::SerializingFactory silver;
    auto file = ReadRiveFile("assets/follow_path_constraint.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi = file->createViewModelInstance(artboard.get());

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);
    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    silver.addFrame();

    stateMachine->advanceAndApply(0.1f);

    artboard->draw(renderer.get());

    int frames = (int)(1.0f / 0.16f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.16f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("follow_path_constraint"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned font_test case 1 awaits typed Rust execution"]
fn wave_b_font_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Inspect Font Styles", "[text_styles]")
{
    struct TestCaseData
    {
        const char* fontPath;
        uint16_t expectedWeight;
        bool expectedItalic;
    };

    std::vector<TestCaseData> testCases = {
        {"assets/fonts/AdventPro-VariableFont_wdth,wght.ttf", 400, false},
        {"assets/fonts/Inter_18pt-Regular.ttf", 400, false},
        {"assets/fonts/Inter_28pt-Bold.ttf", 700, false},
        {"assets/fonts/OpenSans-Italic.ttf", 400, true},
        {"assets/fonts/OpenSans-ExtraBoldItalic.ttf", 800, true},
    };

    for (const auto& testCase : testCases)
    {
        SECTION(testCase.fontPath)
        {
            rive::rcp<Font> font = loadFont(testCase.fontPath);
            HBFont* hbFont = static_cast<HBFont*>(font.get());

            REQUIRE(hbFont->getWeight() == testCase.expectedWeight);
            REQUIRE(hbFont->isItalic() == testCase.expectedItalic);
        }
    }
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned font_test case 2 awaits typed Rust execution"]
fn wave_b_font_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("font exposes cap and x height for vertical trim", "[text_styles]")
{
    // Latin text fonts have an 'H' and 'x'; their tops must sit between the
    // baseline and the ascent, with the x-height below the cap-height. All of
    // capHeight/xHeight/ascent are stored negative (up is -Y).
    std::vector<const char*> fontPaths = {
        "assets/fonts/Inter_18pt-Regular.ttf",
        "assets/Montserrat.ttf",
    };
    for (const char* path : fontPaths)
    {
        SECTION(path)
        {
            rive::rcp<Font> font = loadFont(path);
            const Font::LineMetrics& metrics = font->lineMetrics();

            REQUIRE(metrics.capHeight < 0.0f);
            REQUIRE(metrics.capHeight >= metrics.ascent);
            // x-height is below the cap-height (less far above the baseline).
            REQUIRE(metrics.xHeight > metrics.capHeight);
            REQUIRE(metrics.xHeight < 0.0f);

            // The size-scaled accessors scale linearly.
            REQUIRE(font->capHeight(20.0f) ==
                    Approx(metrics.capHeight * 20.0f));
            REQUIRE(font->xHeight(20.0f) == Approx(metrics.xHeight * 20.0f));
        }
    }
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned font_test case 3 awaits typed Rust execution"]
fn wave_b_font_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("fallback glyphs are found", "[text_fallback]")
{
    REQUIRE(fallbackFonts.empty());
    auto font = loadFont("assets/RobotoFlex.ttf");
    REQUIRE(font != nullptr);
    auto fallbackFont = loadFont("assets/IBMPlexSansArabic-Regular.ttf");
    REQUIRE(fallbackFont != nullptr);
    fallbackFonts.push_back(fallbackFont);

    Font::gFallbackProc = pickFallbackFont;

    std::vector<rive::TextRun> truns;
    std::vector<rive::Unichar> unichars;
    truns.push_back(append(&unichars, font, 32.0f, "لمفاتيح ABC DEF"));

    auto paragraphs = font->shapeText(unichars, truns);
    REQUIRE(paragraphs.size() == 1);
    paragraphs = SimpleArray<Paragraph>();
    REQUIRE(paragraphs.size() == 0);
    fallbackFonts.clear();
    Font::gFallbackProc = nullptr;
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned font_test case 4 awaits typed Rust execution"]
fn wave_b_font_test_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("variable axis values can be read", "[text]")
{
    REQUIRE(fallbackFonts.empty());
    auto font = loadFont("assets/RobotoFlex.ttf");
    REQUIRE(font != nullptr);

    auto count = font->getAxisCount();

    bool hasWeight = false;
    for (uint16_t i = 0; i < count; i++)
    {
        auto axis = font->getAxis(i);
        if (axis.tag == 2003265652)
        {
            REQUIRE(axis.def == 400.0f);
            hasWeight = true;
            break;
        }
    }

    REQUIRE(hasWeight);

    float value = font->getAxisValue(2003265652);
    REQUIRE(value == 400.0f);

    REQUIRE(font->getAxisValue(2003072104) == 100.0f);

    rive::Font::Coord coord = {2003265652, 800.0f};
    rive::rcp<rive::Font> vfont =
        font->makeAtCoords(rive::Span<HBFont::Coord>(&coord, 1));
    REQUIRE(vfont->getAxisValue(2003265652) == 800.0f);

    rive::Font::Coord coord2 = {2003072104, 122.0f};
    rive::rcp<rive::Font> vfont2 =
        vfont->makeAtCoords(rive::Span<HBFont::Coord>(&coord2, 1));
    REQUIRE(vfont2->getAxisValue(2003072104) == 122.0f);
    // Should also still have the first axis value we set.
    REQUIRE(vfont2->getAxisValue(2003265652) == 800.0f);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned font_test case 5 awaits typed Rust execution"]
fn wave_b_font_test_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("font features load as expected", "[text]")
{
    REQUIRE(fallbackFonts.empty());
    auto font = loadFont("assets/RobotoFlex.ttf");
    REQUIRE(font != nullptr);

    rive::SimpleArray<uint32_t> features = font->features();
    std::vector<std::string> featureStrings;
    for (auto feature : features)
    {
        featureStrings.push_back(tagToString(feature));
    }
    REQUIRE(features.size() == 7);

    REQUIRE(hasTag(featureStrings, "mkmk"));
    REQUIRE(hasTag(featureStrings, "kern"));
    REQUIRE(hasTag(featureStrings, "rvrn"));
    REQUIRE(hasTag(featureStrings, "mark"));
    REQUIRE(hasTag(featureStrings, "locl"));
    REQUIRE(hasTag(featureStrings, "pnum"));
    REQUIRE(hasTag(featureStrings, "liga"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned gamepad_test case 1 awaits typed Rust execution"]
fn wave_b_gamepad_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("gamepad batch accepts a single connected record", "[gamepad]")
{
    SerializingFactory silver;
    rcp<File> file;
    std::unique_ptr<ArtboardInstance> artboard;
    auto stateMachine = openReadyStateMachine(file, artboard, silver);

    constexpr int32_t kDeviceId = 0;
    constexpr uint8_t kNButtons = 17;
    constexpr uint8_t kNAxes = 4;
    static_assert(kNButtons <= kGamepadBatchMaxButtons, "button cap");
    static_assert(kNAxes <= kGamepadBatchMaxAxes, "axis cap");

    WireBuilder wb;
    wb.header();
    wb.connected(kDeviceId, kNButtons, kNAxes);

    // Version (4) + record type (1) + deviceId (4) + 4 header bytes
    // + 17 button floats + 4 axis floats.
    REQUIRE(wb.buf.size() ==
            4u + 1u + 4u + 4u + size_t(kNButtons) * 4u + size_t(kNAxes) * 4u);

    CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(), wb.buf.size()));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned gamepad_test case 2 awaits typed Rust execution"]
fn wave_b_gamepad_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("gamepad batch tracks multiple device ids independently", "[gamepad]")
{
    SerializingFactory silver;
    rcp<File> file;
    std::unique_ptr<ArtboardInstance> artboard;
    auto stateMachine = openReadyStateMachine(file, artboard, silver);

    // Connect three devices with distinct ids (including a high id to make
    // sure the int32 path round-trips).
    WireBuilder connect;
    connect.header();
    connect.connected(/*deviceId*/ 1);
    connect.connected(/*deviceId*/ 7);
    connect.connected(/*deviceId*/ 42);
    CHECK(stateMachine->submitGamepadsFromBuffer(connect.buf.data(),
                                                 connect.buf.size()));

    // Send a separate batch per device, mixing button + axis updates.
    WireBuilder updates;
    updates.header();
    updates.updateOne(1, GamepadInputChangeKind::button, /*index*/ 0, 1.f);
    updates.updateOne(7, GamepadInputChangeKind::axis, /*index*/ 2, -0.5f);
    updates.updateOne(42, GamepadInputChangeKind::button, /*index*/ 4, 1.f);
    updates.updateOne(42, GamepadInputChangeKind::axis, /*index*/ 0, 0.75f);
    CHECK(stateMachine->submitGamepadsFromBuffer(updates.buf.data(),
                                                 updates.buf.size()));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned gamepad_test case 3 awaits typed Rust execution"]
fn wave_b_gamepad_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("gamepad batch rejects an update for an unknown device id",
          "[gamepad]")
{
    SerializingFactory silver;
    rcp<File> file;
    std::unique_ptr<ArtboardInstance> artboard;
    auto stateMachine = openReadyStateMachine(file, artboard, silver);

    WireBuilder wb;
    wb.header();
    wb.connected(/*deviceId*/ 3);
    // Update targets a deviceId we never connected — must bail out.
    wb.updateOne(/*deviceId*/ 99,
                 GamepadInputChangeKind::button,
                 /*index*/ 0,
                 1.f);

    CHECK_FALSE(
        stateMachine->submitGamepadsFromBuffer(wb.buf.data(), wb.buf.size()));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned gamepad_test case 4 awaits typed Rust execution"]
fn wave_b_gamepad_test_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("gamepad batch handles disconnect of one of several devices",
          "[gamepad]")
{
    SerializingFactory silver;
    rcp<File> file;
    std::unique_ptr<ArtboardInstance> artboard;
    auto stateMachine = openReadyStateMachine(file, artboard, silver);

    WireBuilder wb;
    wb.header();
    wb.connected(/*deviceId*/ 10);
    wb.connected(/*deviceId*/ 20);
    wb.connected(/*deviceId*/ 30);
    // Drive an update on each, disconnect the middle one, then drive the
    // surviving devices again to confirm they are still tracked.
    wb.updateOne(10, GamepadInputChangeKind::button, 0, 1.f);
    wb.updateOne(20, GamepadInputChangeKind::axis, 1, 0.25f);
    wb.updateOne(30, GamepadInputChangeKind::button, 2, 1.f);
    wb.disconnected(/*deviceId*/ 20);
    wb.updateOne(10, GamepadInputChangeKind::axis, 0, -1.f);
    wb.updateOne(30, GamepadInputChangeKind::button, 3, 0.f);

    CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(), wb.buf.size()));

    // Anything addressed to the now-disconnected device must be rejected.
    WireBuilder afterDisconnect;
    afterDisconnect.header();
    afterDisconnect.updateOne(/*deviceId*/ 20,
                              GamepadInputChangeKind::button,
                              0,
                              1.f);
    CHECK_FALSE(
        stateMachine->submitGamepadsFromBuffer(afterDisconnect.buf.data(),
                                               afterDisconnect.buf.size()));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned gamepad_test case 5 awaits typed Rust execution"]
fn wave_b_gamepad_test_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("gamepad batch allows reconnecting the same device id", "[gamepad]")
{
    SerializingFactory silver;
    rcp<File> file;
    std::unique_ptr<ArtboardInstance> artboard;
    auto stateMachine = openReadyStateMachine(file, artboard, silver);

    constexpr int32_t kDeviceId = 5;

    WireBuilder first;
    first.header();
    first.connected(kDeviceId);
    first.updateOne(kDeviceId, GamepadInputChangeKind::button, 0, 1.f);
    first.disconnected(kDeviceId);
    CHECK(stateMachine->submitGamepadsFromBuffer(first.buf.data(),
                                                 first.buf.size()));

    // After disconnect the device must be unknown again — any update before a
    // fresh connect is rejected.
    WireBuilder strayUpdate;
    strayUpdate.header();
    strayUpdate.updateOne(kDeviceId, GamepadInputChangeKind::button, 0, 1.f);
    CHECK_FALSE(stateMachine->submitGamepadsFromBuffer(strayUpdate.buf.data(),
                                                       strayUpdate.buf.size()));

    // Reconnect with the same id and confirm updates flow again.
    WireBuilder reconnect;
    reconnect.header();
    reconnect.connected(kDeviceId);
    reconnect.updateOne(kDeviceId, GamepadInputChangeKind::axis, 0, 0.5f);
    CHECK(stateMachine->submitGamepadsFromBuffer(reconnect.buf.data(),
                                                 reconnect.buf.size()));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned gamepad_test case 6 awaits typed Rust execution"]
fn wave_b_gamepad_test_006_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("gamepad batch tolerates disconnect of an unknown device id",
          "[gamepad]")
{
    SerializingFactory silver;
    rcp<File> file;
    std::unique_ptr<ArtboardInstance> artboard;
    auto stateMachine = openReadyStateMachine(file, artboard, silver);

    // Disconnect for a device we never connected is a no-op `erase` per the
    // wire-format contract — the batch must still parse successfully.
    WireBuilder wb;
    wb.header();
    wb.disconnected(/*deviceId*/ 1234);

    CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(), wb.buf.size()));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned gamepad_test case 7 awaits typed Rust execution"]
fn wave_b_gamepad_test_007_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("File loads and processes multiple types of gamepad inputs",
          "[gamepad]")
{
    SerializingFactory silver;
    rcp<File> file;
    std::unique_ptr<ArtboardInstance> artboard;
    auto stateMachine = openReadyStateMachine(file, artboard, silver);

    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    stateMachine->advanceAndApply(0.1f);
    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());
    silver.addFrame();

    constexpr int32_t kDeviceId1 = 3;
    constexpr int32_t kDeviceId2 = 5;
    constexpr int32_t kDeviceId3 = 1;

    // connect device 1
    {
        WireBuilder wb;
        wb.header();
        wb.connected(kDeviceId1);
        CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(),
                                                     wb.buf.size()));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    // press button at index 0 on device 1
    {

        silver.addFrame();
        WireBuilder wb;
        wb.header();
        wb.updateOne(kDeviceId1, GamepadInputChangeKind::button, 0, 1.f);
        CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(),
                                                     wb.buf.size()));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }
    // connect device 2 and 3
    {
        silver.addFrame();
        WireBuilder wb;
        wb.header();
        wb.connected(kDeviceId2);
        wb.connected(kDeviceId3,
                     17,
                     4,
                     1); // Device 3 is not a standard mapping
        CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(),
                                                     wb.buf.size()));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }
    // submit button press on device 2 and 3
    {
        silver.addFrame();
        WireBuilder wb;
        wb.header();
        wb.updateOne(kDeviceId2, GamepadInputChangeKind::button, 2, 1.f);
        wb.updateOne(kDeviceId3, GamepadInputChangeKind::button, 2, 1.f);
        CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(),
                                                     wb.buf.size()));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }
    // submit axis change on device 1 and 2
    {

        for (auto i = 1; i < 10; i++)
        {
            silver.addFrame();
            WireBuilder wb;
            wb.header();
            wb.updateOne(kDeviceId1, GamepadInputChangeKind::axis, 0, i * 0.1f);
            wb.updateOne(kDeviceId2,
                         GamepadInputChangeKind::axis,
                         1,
                         i * -0.1f);
            CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(),
                                                         wb.buf.size()));
            stateMachine->advanceAndApply(0.016f);
            artboard->draw(renderer.get());
        }
    }
    // disconnect device 3
    {
        silver.addFrame();
        WireBuilder wb;
        wb.header();
        wb.disconnected(kDeviceId3);
        CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(),
                                                     wb.buf.size()));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }
    // release button 0 of device 1
    {
        stateMachine->focusManager()->focusNext();
        silver.addFrame();
        WireBuilder wb;
        wb.header();
        wb.updateOne(kDeviceId1, GamepadInputChangeKind::button, 0, 0.f);
        CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(),
                                                     wb.buf.size()));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }
    // press button 0 of device 1
    {
        silver.addFrame();
        WireBuilder wb;
        wb.header();
        wb.updateOne(kDeviceId1, GamepadInputChangeKind::button, 0, 1.f);
        CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(),
                                                     wb.buf.size()));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }
    // Release button 0
    {
        silver.addFrame();
        WireBuilder wb;
        wb.header();
        wb.updateOne(kDeviceId1, GamepadInputChangeKind::button, 0, 0.f);
        CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(),
                                                     wb.buf.size()));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }
    {
        silver.addFrame();
        WireBuilder wb;
        wb.header();
        wb.updateOne(kDeviceId1, GamepadInputChangeKind::button, 1, 1.f);
        CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(),
                                                     wb.buf.size()));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }
    // Move left joystick x and y axis
    {
        silver.addFrame();
        WireBuilder wb;
        wb.header();
        wb.updateOne(kDeviceId1, GamepadInputChangeKind::axis, 0, 0.5f);
        wb.updateOne(kDeviceId1, GamepadInputChangeKind::axis, 1, 0.5f);
        CHECK(stateMachine->submitGamepadsFromBuffer(wb.buf.data(),
                                                     wb.buf.size()));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("gamepad_test"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 1 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("File::globalViewModelNames lists globals in file order",
          "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto names = file->globalViewModelNames();
    REQUIRE_FALSE(names.empty());
    // Every listed name resolves to a global view model.
    for (auto& name : names)
    {
        auto vm = file->viewModel(name);
        REQUIRE(vm != nullptr);
        REQUIRE(static_cast<ViewModelType>(vm->viewModelType()) ==
                ViewModelType::global);
    }
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 2 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("artboard instancing does not auto-create globals", "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    REQUIRE(artboard->dataContext() == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 3 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("global getter is null until set", "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    auto names = file->globalViewModelNames();
    REQUIRE_FALSE(names.empty());
    const std::string target = names[0];

    REQUIRE(artboard->globalViewModelInstance(target) == nullptr);

    auto instance =
        file->createDefaultViewModelInstance(file->viewModel(target));
    REQUIRE(instance != nullptr);
    REQUIRE(artboard->setGlobalViewModelInstance(target, instance));
    REQUIRE(artboard->globalViewModelInstance(target) == instance);

    // A non-global name is rejected and stays null.
    REQUIRE_FALSE(
        artboard->setGlobalViewModelInstance("not-a-global", instance));
    REQUIRE(artboard->globalViewModelInstance("not-a-global") == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 4 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("set without bind mutates order; bind applies", "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    auto globalNames = file->globalViewModelNames();
    REQUIRE_FALSE(globalNames.empty());

    auto mainVmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(mainVmi != nullptr);
    auto mainVm = mainVmi->viewModel();
    REQUIRE(mainVm != nullptr);
    REQUIRE(static_cast<ViewModelType>(mainVm->viewModelType()) !=
            ViewModelType::global);

    // Batch: set main + each global, then a single bind().
    artboard->setViewModelInstance(mainVmi);
    for (auto& name : globalNames)
    {
        auto g = file->createDefaultViewModelInstance(file->viewModel(name));
        REQUIRE(artboard->setGlobalViewModelInstance(name, g));
    }

    // The data context already reflects the sets (getter reads pre-bind).
    std::vector<std::string> expected;
    expected.push_back(mainVm->name());
    for (auto& n : globalNames)
    {
        expected.push_back(n);
    }
    REQUIRE(boundNames(artboard->dataContext().get()) == expected);

    artboard->bind();
    // Order is unchanged after applying.
    REQUIRE(boundNames(artboard->dataContext().get()) == expected);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 5 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("globals are ordered by file definition regardless of set order",
          "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    auto globalNames = file->globalViewModelNames();
    // Need at least two globals to observe ordering.
    REQUIRE(globalNames.size() >= 2);

    // Set them in reverse file order.
    for (auto it = globalNames.rbegin(); it != globalNames.rend(); ++it)
    {
        auto g = file->createDefaultViewModelInstance(file->viewModel(*it));
        REQUIRE(artboard->setGlobalViewModelInstance(*it, g));
    }

    // The data context still lists them in file order.
    REQUIRE(boundNames(artboard->dataContext().get()) == globalNames);

    // Setting a main afterwards keeps globals ordered, main first.
    auto mainVmi = file->createDefaultViewModelInstance(artboard.get());
    if (mainVmi != nullptr &&
        static_cast<ViewModelType>(mainVmi->viewModel()->viewModelType()) !=
            ViewModelType::global)
    {
        artboard->setViewModelInstance(mainVmi);
        std::vector<std::string> expected;
        expected.push_back(mainVmi->viewModel()->name());
        for (auto& n : globalNames)
        {
            expected.push_back(n);
        }
        REQUIRE(boundNames(artboard->dataContext().get()) == expected);
    }
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 6 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_006_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("bind completes missing global slots on the fly", "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);

    auto globalNames = file->globalViewModelNames();
    REQUIRE_FALSE(globalNames.empty());

    // Bind only a main — no globals set explicitly.
    auto mainVmi = file->createDefaultViewModelInstance(artboard.get());
    REQUIRE(mainVmi != nullptr);
    stateMachine->bindViewModelInstance(mainVmi);

    // Every global slot has been completed by bind().
    for (auto& name : globalNames)
    {
        REQUIRE(stateMachine->globalViewModelInstance(name) != nullptr);
    }
    auto dc = stateMachine->dataContext();
    REQUIRE(dc != nullptr);
    // [main, globals...] — main plus one instance per global slot.
    REQUIRE(boundNames(dc.get()).size() == globalNames.size() + 1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 7 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_007_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("slot accepts a different view model instance (override)",
          "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    auto globalNames = file->globalViewModelNames();
    REQUIRE(globalNames.size() >= 2);

    const std::string slotA = globalNames[0];
    const std::string vmB = globalNames[1];

    // An instance of view model B, placed onto slot A.
    auto override = file->createDefaultViewModelInstance(file->viewModel(vmB));
    REQUIRE(override != nullptr);
    REQUIRE(override->viewModel()->name() == vmB);

    // Previously rejected (name != instance's VM); now accepted.
    REQUIRE(artboard->setGlobalViewModelInstance(slotA, override));

    // Slot A resolves to the B-typed instance (addressed by slot, not by VM).
    REQUIRE(artboard->globalViewModelInstance(slotA) == override);
    // Occupancy is keyed by slot: B's own slot stays empty — the instance's own
    // viewModelId does not place it into B's slot.
    REQUIRE(artboard->globalViewModelInstance(vmB) == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 8 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_008_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("state machine set/bind and get by name", "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    auto globalNames = file->globalViewModelNames();
    REQUIRE_FALSE(globalNames.empty());

    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);

    // Null until set.
    REQUIRE(stateMachine->globalViewModelInstance(globalNames[0]) == nullptr);

    auto mainVmi = file->createDefaultViewModelInstance(artboard.get());
    stateMachine->setViewModelInstance(mainVmi);
    for (auto& name : globalNames)
    {
        auto g = file->createDefaultViewModelInstance(file->viewModel(name));
        REQUIRE(stateMachine->setGlobalViewModelInstance(name, g));
    }
    stateMachine->bind();

    auto dc = stateMachine->dataContext();
    REQUIRE(dc != nullptr);
    auto names = boundNames(dc.get());
    REQUIRE(names.size() == globalNames.size() + 1);
    REQUIRE(names[0] == mainVmi->viewModel()->name());

    auto fetched = stateMachine->globalViewModelInstance(globalNames[0]);
    REQUIRE(fetched != nullptr);
    REQUIRE(fetched->viewModel()->name() == globalNames[0]);

    // Replacing by name preserves position.
    auto custom =
        file->createDefaultViewModelInstance(file->viewModel(globalNames[0]));
    REQUIRE(stateMachine->setGlobalViewModelInstance(globalNames[0], custom));
    REQUIRE(stateMachine->globalViewModelInstance(globalNames[0]) == custom);
    REQUIRE(boundNames(stateMachine->dataContext().get()) == names);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 9 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_009_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("setGlobalViewModelInstance rejects a non-global name", "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);

    // Find a real view model that is not a global.
    std::string nonGlobal;
    for (size_t i = 0; i < file->viewModelCount(); i++)
    {
        auto vm = file->viewModel(i);
        if (vm != nullptr && static_cast<ViewModelType>(vm->viewModelType()) !=
                                 ViewModelType::global)
        {
            nonGlobal = vm->name();
            break;
        }
    }
    REQUIRE_FALSE(nonGlobal.empty());

    auto instance =
        file->createDefaultViewModelInstance(file->viewModel(nonGlobal));
    REQUIRE(instance != nullptr);

    // Rejected on both the artboard and the state machine paths; no slot
    // filled.
    REQUIRE_FALSE(artboard->setGlobalViewModelInstance(nonGlobal, instance));
    REQUIRE(artboard->globalViewModelInstance(nonGlobal) == nullptr);
    REQUIRE_FALSE(
        stateMachine->setGlobalViewModelInstance(nonGlobal, instance));
    REQUIRE(stateMachine->globalViewModelInstance(nonGlobal) == nullptr);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 10 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_010_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("bind creates a data context when none is set", "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);

    auto globalNames = file->globalViewModelNames();
    REQUIRE_FALSE(globalNames.empty());

    // Nothing has been set: the state machine has no data context.
    REQUIRE(stateMachine->dataContext() == nullptr);

    stateMachine->bind();

    // A context now exists, holding a main plus one instance per global slot.
    auto dc = stateMachine->dataContext();
    REQUIRE(dc != nullptr);
    REQUIRE(boundNames(dc.get()).size() == globalNames.size() + 1);
    for (auto& name : globalNames)
    {
        REQUIRE(stateMachine->globalViewModelInstance(name) != nullptr);
    }
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 11 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_011_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("setGlobalViewModelInstance with null empties the slot",
          "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);

    auto globalNames = file->globalViewModelNames();
    // Need at least two globals to prove the others are untouched.
    REQUIRE(globalNames.size() >= 2);

    for (auto& name : globalNames)
    {
        auto g = file->createDefaultViewModelInstance(file->viewModel(name));
        REQUIRE(stateMachine->setGlobalViewModelInstance(name, g));
    }
    REQUIRE(stateMachine->globalViewModelInstance(globalNames[0]) != nullptr);
    REQUIRE(boundNames(stateMachine->dataContext().get()).size() ==
            globalNames.size());

    // Empty the first slot with a null instance.
    REQUIRE(stateMachine->setGlobalViewModelInstance(globalNames[0], nullptr));

    // That slot now reads as unset; every other slot is untouched.
    REQUIRE(stateMachine->globalViewModelInstance(globalNames[0]) == nullptr);
    for (size_t i = 1; i < globalNames.size(); i++)
    {
        REQUIRE(stateMachine->globalViewModelInstance(globalNames[i]) !=
                nullptr);
    }
    REQUIRE(boundNames(stateMachine->dataContext().get()).size() ==
            globalNames.size() - 1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 12 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_012_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("bind adds a main when only a global is set", "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);

    auto globalNames = file->globalViewModelNames();
    REQUIRE_FALSE(globalNames.empty());

    // Set a single global, no main.
    auto g =
        file->createDefaultViewModelInstance(file->viewModel(globalNames[0]));
    REQUIRE(g != nullptr);
    REQUIRE(stateMachine->setGlobalViewModelInstance(globalNames[0], g));

    // Pre-bind: a context exists but has no main instance.
    auto dc = stateMachine->dataContext();
    REQUIRE(dc != nullptr);
    REQUIRE(dc->mainViewModelInstance() == nullptr);

    stateMachine->bind();

    // bind() completed the main; it leads the [main, globals...] order.
    REQUIRE(dc->mainViewModelInstance() != nullptr);
    auto names = boundNames(dc.get());
    REQUIRE(names.size() == globalNames.size() + 1);
    REQUIRE(names[0] == dc->mainViewModelInstance()->viewModel()->name());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 13 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_013_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("setGlobalViewModelInstance null on empty context is a no-op",
          "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    REQUIRE(stateMachine != nullptr);

    auto globalNames = file->globalViewModelNames();
    REQUIRE_FALSE(globalNames.empty());

    REQUIRE(stateMachine->dataContext() == nullptr);

    // Clearing an already-empty slot succeeds without allocating a context.
    REQUIRE(stateMachine->setGlobalViewModelInstance(globalNames[0], nullptr));
    REQUIRE(stateMachine->dataContext() == nullptr);

    // A non-global name is still rejected, even with a null instance.
    REQUIRE_FALSE(
        stateMachine->setGlobalViewModelInstance("not-a-global", nullptr));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 14 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_014_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("artboard setGlobalViewModelInstance with null empties the slot",
          "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    auto globalNames = file->globalViewModelNames();
    // Need at least two globals to prove the others are untouched.
    REQUIRE(globalNames.size() >= 2);

    for (auto& name : globalNames)
    {
        auto g = file->createDefaultViewModelInstance(file->viewModel(name));
        REQUIRE(artboard->setGlobalViewModelInstance(name, g));
    }
    REQUIRE(artboard->globalViewModelInstance(globalNames[0]) != nullptr);
    REQUIRE(boundNames(artboard->dataContext().get()).size() ==
            globalNames.size());

    // Empty the first slot with a null instance.
    REQUIRE(artboard->setGlobalViewModelInstance(globalNames[0], nullptr));

    // That slot now reads as unset; every other slot is untouched.
    REQUIRE(artboard->globalViewModelInstance(globalNames[0]) == nullptr);
    for (size_t i = 1; i < globalNames.size(); i++)
    {
        REQUIRE(artboard->globalViewModelInstance(globalNames[i]) != nullptr);
    }
    REQUIRE(boundNames(artboard->dataContext().get()).size() ==
            globalNames.size() - 1);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_view_model_binding_test case 15 awaits typed Rust execution"]
fn wave_b_global_view_model_binding_test_015_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("artboard setGlobalViewModelInstance null on empty context no-ops",
          "[viewmodel]")
{
    auto file = ReadRiveFile("assets/global_variables_test.riv");
    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    auto globalNames = file->globalViewModelNames();
    REQUIRE_FALSE(globalNames.empty());

    REQUIRE(artboard->dataContext() == nullptr);

    // Clearing an already-empty slot succeeds without allocating a context.
    REQUIRE(artboard->setGlobalViewModelInstance(globalNames[0], nullptr));
    REQUIRE(artboard->dataContext() == nullptr);

    // A non-global name is still rejected, even with a null instance.
    REQUIRE_FALSE(
        artboard->setGlobalViewModelInstance("not-a-global", nullptr));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_viewmodels_test case 1 awaits typed Rust execution"]
fn wave_b_global_viewmodels_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Global view models and overrides", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/global_variables_test.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());

    // Globals are no longer auto-created at instance time; create + set a
    // default for each (as the high-level runtime's autoBind does), then apply
    // everything with a single bind.
    stateMachine->setViewModelInstance(vmi);
    for (auto& name : file->globalViewModelNames())
    {
        auto global =
            file->createDefaultViewModelInstance(file->viewModel(name));
        REQUIRE(global != nullptr);
        REQUIRE(stateMachine->setGlobalViewModelInstance(name, global));
    }
    stateMachine->bind();
    stateMachine->advanceAndApply(0.1f);
    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());
    int frames = (int)(1.0f / 0.016f);
    for (int i = 0; i < frames; i++)
    {
        silver.addFrame();
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }
    CHECK(silver.matches("global_variables_test"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_viewmodels_test case 2 awaits typed Rust execution"]
fn wave_b_global_viewmodels_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Test global view models with automatic instancing", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/global_viewmodels_test.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());

    auto renderer = silver.makeRenderer();
    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("global_viewmodels_test-auto_instance"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned global_viewmodels_test case 3 awaits typed Rust execution"]
fn wave_b_global_viewmodels_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Test global view models with instance explicitly specified",
          "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/global_viewmodels_test.riv", &silver);

    auto artboard = file->artboardDefault();
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    auto renderer = silver.makeRenderer();

    {
        auto vmi = file->createDefaultViewModelInstance(artboard.get());
        auto globalColorsVM = file->viewModel("GlobalColors");
        auto vmiColors = file->createDefaultViewModelInstance(globalColorsVM);
        auto c1Prop =
            vmiColors->propertyValue("c1")->as<ViewModelInstanceColor>();

        auto yellowColor = (255 << 24) | (255 << 16) | (255 << 8);
        c1Prop->propertyValue(yellowColor);

        stateMachine->setViewModelInstance(vmi);
        stateMachine->setGlobalViewModelInstance("GlobalColors", vmiColors);
        stateMachine->bind();
    }

    stateMachine->advanceAndApply(0.0f);
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    {
        auto vmi = file->createDefaultViewModelInstance(artboard.get());
        auto labelProp =
            vmi->propertyValue("label")->as<ViewModelInstanceString>();
        labelProp->propertyValue("label updated");
        auto globalColorsVM = file->viewModel("GlobalColors");
        auto vmiColors = file->createDefaultViewModelInstance(globalColorsVM);
        auto c1Prop =
            vmiColors->propertyValue("c1")->as<ViewModelInstanceColor>();

        auto cyanColor = (255 << 24) | (255 << 8) | 255;
        c1Prop->propertyValue(cyanColor);

        stateMachine->setGlobalViewModelInstance("GlobalColors", vmiColors);
        stateMachine->setViewModelInstance(vmi);
        stateMachine->bind();
    }
    silver.addFrame();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("global_viewmodels_test-set_instance"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 1 awaits typed Rust execution"]
fn wave_b_hittest_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("hittest-basics", "[hittest]")
{
    HitTester tester;
    tester.reset({10, 10, 12, 12});
    tester.move({0, 0});
    tester.line({20, 0});
    tester.line({20, 20});
    tester.line({0, 20});
    tester.close();
    REQUIRE(tester.test());

    IAABB area = {81, 156, 84, 159};

    Vec2D pts[] = {
        {29.9785f, 32.5261f},
        {231.102f, 32.5261f},
        {231.102f, 269.898f},
        {29.9785f, 269.898f},
    };
    tester.reset(area);

    tester.move(pts[0]);
    for (int i = 1; i < 4; ++i)
    {
        tester.line(pts[i]);
    }
    tester.close();
    REQUIRE(tester.test());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 2 awaits typed Rust execution"]
fn wave_b_hittest_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("hittest-mesh", "[hittest]")
{

    const IAABB area{10, 10, 12, 12};

    Vec2D verts[] = {
        {0, 0},
        {20, 10},
        {0, 20},
    };
    uint16_t indices[] = {
        0,
        1,
        2,
    };
    REQUIRE(
        HitTester::testMesh(area, make_span(verts, 3), make_span(indices, 3)));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 3 awaits typed Rust execution"]
fn wave_b_hittest_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("hit test on opaque target", "[hittest]")
{
    // This artboard has two rects of size 200 x 200, "red-activate" at [0, 0,
    // 200, 200] and "green-activate" at [0, 100, 200, 300] "red-activate" is
    // above "green-activate" in drawing order Both targets are set as opaque
    // for its listeners "red-activate" sets "toGreen" to false "green-activate"
    // sets "toGreen" to true There is also a "gray-activate" above the other 2
    // that is not opaque so events should traverse through the other targets
    auto file = ReadRiveFile("assets/opaque_hit_test.riv");

    auto artboard = file->artboard("main");
    auto artboardInstance = artboard->instance();
    auto stateMachine = artboard->stateMachine("main-state-machine");

    REQUIRE(artboardInstance != nullptr);
    REQUIRE(artboardInstance->stateMachineCount() == 1);

    REQUIRE(stateMachine != nullptr);

    rive::StateMachineInstance* stateMachineInstance =
        new rive::StateMachineInstance(stateMachine, artboardInstance.get());

    stateMachineInstance->advance(0.0f);
    artboardInstance->advance(0.0f);
    REQUIRE(stateMachineInstance->needsAdvance() == true);
    stateMachineInstance->advance(0.0f);

    auto toGreenToggle = stateMachineInstance->getBool("toGreen");
    REQUIRE(toGreenToggle != nullptr);
    auto grayToggle = stateMachineInstance->getBool("grayToggle");
    REQUIRE(grayToggle != nullptr);

    stateMachineInstance->pointerDown(rive::Vec2D(100.0f, 50.0f));
    // "gray-activate" is clicked
    REQUIRE(grayToggle->value() == true);
    // Pointer only over "red-activate"
    REQUIRE(toGreenToggle->value() == false);

    stateMachineInstance->pointerDown(rive::Vec2D(100.0f, 250.0f));
    // "gray-activate" is clicked
    REQUIRE(grayToggle->value() == false);
    // Pointer over "green-activate"
    REQUIRE(toGreenToggle->value() == true);

    stateMachineInstance->pointerDown(rive::Vec2D(100.0f, 110.0f));
    // "gray-activate" is clicked
    REQUIRE(grayToggle->value() == true);
    // Pointer over "red-activate" and "green-activate", but "red-activate" is
    // opaque and above so green activate does not trigger
    REQUIRE(toGreenToggle->value() == false);
    delete stateMachineInstance;
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 4 awaits typed Rust execution"]
fn wave_b_hittest_test_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("hit test on opaque nested artboard", "[hittest]")
{
    // This artboard (300x300) has a main rect at [0, 0, 300, 300]
    // this rect has a listener that toggles "second-gray-toggle"
    // and a nested artboard at [0, 0, 150, 150]
    // the nested artboard and the rect have opaque targets
    auto file = ReadRiveFile("assets/opaque_hit_test.riv");

    auto artboard = file->artboard("second");
    auto artboardInstance = artboard->instance();
    auto stateMachine = artboard->stateMachine("second-state-machine");

    REQUIRE(artboardInstance != nullptr);
    REQUIRE(artboardInstance->stateMachineCount() == 1);

    REQUIRE(stateMachine != nullptr);

    rive::StateMachineInstance* stateMachineInstance =
        new rive::StateMachineInstance(stateMachine, artboardInstance.get());

    auto nestedArtboard =
        stateMachineInstance->artboard()->find<rive::NestedArtboard>(
            "second-nested");
    REQUIRE(nestedArtboard != nullptr);
    auto nestedArtboardStateMachine =
        nestedArtboard->nestedAnimations()[0]->as<NestedStateMachine>();
    REQUIRE(nestedArtboardStateMachine != nullptr);
    auto nestedArtboardStateMachineInstance =
        nestedArtboardStateMachine->stateMachineInstance();

    auto secondNestedBoolTarget =
        nestedArtboardStateMachineInstance->getBool("bool-target");
    REQUIRE(secondNestedBoolTarget != nullptr);

    artboardInstance->advance(0.0f);
    stateMachineInstance->advanceAndApply(0.0f);

    REQUIRE(secondNestedBoolTarget->value() == false);

    auto secondGrayToggle = stateMachineInstance->getBool("second-gray-toggle");
    REQUIRE(secondGrayToggle != nullptr);

    stateMachineInstance->pointerDown(rive::Vec2D(100.0f, 250.0f));
    // toggle changes value because it is not under an opaque nested artboard
    REQUIRE(secondGrayToggle->value() == true);

    stateMachineInstance->pointerDown(rive::Vec2D(301.0f, 50.0f));
    // toggle does not change because it is beyond the area of the square by 1
    // pixel And the 2px padding is unly used after the coarse grained test
    // passes
    REQUIRE(secondGrayToggle->value() == true);

    stateMachineInstance->pointerDown(rive::Vec2D(100.0f, 50.0f));
    // toggle does not change because it is under an opaque nested artboard
    REQUIRE(secondGrayToggle->value() == true);

    // nested toggle changes because it's on top of shape
    REQUIRE(secondNestedBoolTarget->value() == true);

    // A timeline switches draw order and the nested artboard is now below the
    // rect
    stateMachineInstance->advanceAndApply(1.0f);
    stateMachineInstance->advance(0.0f);

    stateMachineInstance->pointerDown(rive::Vec2D(100.0f, 50.0f));
    // So now the pointer down is captured by the rect
    REQUIRE(secondGrayToggle->value() == false);

    // nested toggle does not change because it's below shape
    REQUIRE(secondNestedBoolTarget->value() == true);
    delete stateMachineInstance;
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 5 awaits typed Rust execution"]
fn wave_b_hittest_test_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("early out on listeners", "[hittest]")
{
    auto file = ReadRiveFile("assets/pointer_events.riv");

    auto artboard = file->artboard("art-1");
    auto artboardInstance = artboard->instance();
    auto stateMachine = artboard->stateMachine("sm-1");

    REQUIRE(artboardInstance != nullptr);
    REQUIRE(artboardInstance->stateMachineCount() == 1);

    REQUIRE(stateMachine != nullptr);

    rive::StateMachineInstance* stateMachineInstance =
        new rive::StateMachineInstance(stateMachine, artboardInstance.get());

    stateMachineInstance->advance(0.0f);
    artboardInstance->advance(0.0f);
    REQUIRE(stateMachineInstance->needsAdvance() == true);
    stateMachineInstance->advance(0.0f);
    REQUIRE(stateMachineInstance->hitComponentsCount() == 4);
    // Hit component with only pointer down and pointer up listeners
    auto hitComponentWithEarlyOut = stateMachineInstance->hitComponent(0);
    // Hit component that can't early out because it has a pointer enter event
    auto hitComponentWithNoEarlyOut = stateMachineInstance->hitComponent(1);
    // Hit component that can't early out because it is an opaque target
    auto hitComponentOpaque = stateMachineInstance->hitComponent(2);
    // Hit component that can early out on all and pointer up
    auto hitComponentOnlyPointerDown = stateMachineInstance->hitComponent(3);
    REQUIRE(hitComponentWithEarlyOut->earlyOutCount == 0);
    REQUIRE(hitComponentWithNoEarlyOut->earlyOutCount == 0);
    REQUIRE(hitComponentOpaque->earlyOutCount == 0);
    REQUIRE(hitComponentOnlyPointerDown->earlyOutCount == 0);
    stateMachineInstance->pointerMove(rive::Vec2D(100.0f, 250.0f));
    REQUIRE(hitComponentWithEarlyOut->earlyOutCount == 1);
    REQUIRE(hitComponentWithNoEarlyOut->earlyOutCount == 0);
    REQUIRE(hitComponentOpaque->earlyOutCount == 0);
    REQUIRE(hitComponentOnlyPointerDown->earlyOutCount == 1);
    stateMachineInstance->pointerExit(rive::Vec2D(100.0f, 250.0f));
    REQUIRE(hitComponentWithEarlyOut->earlyOutCount == 2);
    REQUIRE(hitComponentWithNoEarlyOut->earlyOutCount == 0);
    REQUIRE(hitComponentOnlyPointerDown->earlyOutCount == 2);
    REQUIRE(hitComponentOpaque->earlyOutCount == 0);
    stateMachineInstance->pointerDown(rive::Vec2D(100.0f, 250.0f));
    REQUIRE(hitComponentWithEarlyOut->earlyOutCount == 2);
    REQUIRE(hitComponentWithNoEarlyOut->earlyOutCount == 0);
    REQUIRE(hitComponentOpaque->earlyOutCount == 0);
    REQUIRE(hitComponentOnlyPointerDown->earlyOutCount == 2);
    stateMachineInstance->pointerUp(rive::Vec2D(100.0f, 250.0f));
    REQUIRE(hitComponentWithEarlyOut->earlyOutCount == 2);
    REQUIRE(hitComponentWithNoEarlyOut->earlyOutCount == 0);
    REQUIRE(hitComponentOpaque->earlyOutCount == 0);
    REQUIRE(hitComponentOnlyPointerDown->earlyOutCount == 3);
    stateMachineInstance->pointerMove(rive::Vec2D(105.0f, 205.0f));
    REQUIRE(hitComponentWithEarlyOut->earlyOutCount == 3);
    REQUIRE(hitComponentWithNoEarlyOut->earlyOutCount == 0);
    REQUIRE(hitComponentOpaque->earlyOutCount == 0);
    REQUIRE(hitComponentOnlyPointerDown->earlyOutCount == 4);

    delete stateMachineInstance;
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 6 awaits typed Rust execution"]
fn wave_b_hittest_test_006_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("click event", "[hittest]")
{
    // This test has two rectangles of size [200, 200]
    // positioned at [100,100] and [200, 200]
    // they overlap between coordinates [100,100]-[200, 200]
    // they are inside a group that has a listener attached to it
    // that listener should fire an event on "Click"
    auto file = ReadRiveFile("assets/click_event.riv");

    auto artboard = file->artboard("art-1");
    auto artboardInstance = artboard->instance();
    auto stateMachine = artboard->stateMachine("sm-1");

    REQUIRE(artboardInstance != nullptr);
    REQUIRE(artboardInstance->stateMachineCount() == 1);

    REQUIRE(stateMachine != nullptr);

    rive::StateMachineInstance* stateMachineInstance =
        new rive::StateMachineInstance(stateMachine, artboardInstance.get());

    stateMachineInstance->advance(0.0f);
    artboardInstance->advance(0.0f);
    REQUIRE(stateMachineInstance->needsAdvance() == true);
    stateMachineInstance->advance(0.0f);
    // There is a single listener with two shapes in it
    REQUIRE(stateMachineInstance->hitComponentsCount() == 2);
    auto layerCount = stateMachine->layerCount();
    REQUIRE(layerCount == 1);
    REQUIRE(stateMachineInstance->reportedEventCount() == 0);
    // Click in place should trigger a click event
    stateMachineInstance->pointerDown(rive::Vec2D(75.0f, 75.0f));
    stateMachineInstance->pointerUp(rive::Vec2D(75.0f, 75.0f));
    REQUIRE(stateMachineInstance->reportedEventCount() == 1);
    // Pointer down inside shape but Pointer up outside the shape
    // should not trigger a click event
    stateMachineInstance->pointerDown(rive::Vec2D(75.0f, 75.0f));
    stateMachineInstance->pointerUp(rive::Vec2D(300.0f, 75.0f));
    REQUIRE(stateMachineInstance->reportedEventCount() == 1);
    // Pointer down outside shape but Pointer up inside the shape
    // should not trigger a click event
    stateMachineInstance->pointerDown(rive::Vec2D(300.0f, 75.0f));
    stateMachineInstance->pointerUp(rive::Vec2D(75.0f, 75.0f));
    REQUIRE(stateMachineInstance->reportedEventCount() == 1);
    // Pointer down in shape 1 Pointer up in shape 2 of the same group
    // should trigger a click event
    stateMachineInstance->pointerDown(rive::Vec2D(75.0f, 75.0f));
    stateMachineInstance->pointerUp(rive::Vec2D(225.0f, 225.0f));
    REQUIRE(stateMachineInstance->reportedEventCount() == 2);
    // Pointer down and up in area where both shapes overlap
    // should trigger a single click event
    stateMachineInstance->pointerDown(rive::Vec2D(150.0f, 150.0f));
    stateMachineInstance->pointerUp(rive::Vec2D(150.0f, 150.0f));
    REQUIRE(stateMachineInstance->reportedEventCount() == 3);

    delete stateMachineInstance;
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 7 awaits typed Rust execution"]
fn wave_b_hittest_test_007_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("multiple shapes with mouse movement behavior", "[hittest]")
{
    // This test has two rectangles of size [200, 200]
    // positioned at [100,100] and [100, 200]
    // they overlap between coordinates [100,0]-[200, 200]
    // they are inside a group that has a Pointer enter and a Pointer out
    // listeners that toggle between two states (red and green)
    // starting at "red"
    auto file = ReadRiveFile("assets/click_event.riv");

    auto artboard = file->artboard("art-2");
    auto artboardInstance = artboard->instance();
    auto stateMachine = artboard->stateMachine("sm-1");

    REQUIRE(artboardInstance != nullptr);
    REQUIRE(artboardInstance->stateMachineCount() == 1);

    REQUIRE(stateMachine != nullptr);

    rive::StateMachineInstance* stateMachineInstance =
        new rive::StateMachineInstance(stateMachine, artboardInstance.get());

    stateMachineInstance->advance(0.0f);
    artboardInstance->advance(0.0f);
    REQUIRE(stateMachineInstance->needsAdvance() == true);
    stateMachineInstance->advance(0.0f);
    // There is a single listener with two shapes in it
    REQUIRE(stateMachineInstance->hitComponentsCount() == 2);
    auto layerCount = stateMachine->layerCount();
    REQUIRE(layerCount == 1);
    // Move over the first shape
    stateMachineInstance->pointerMove(rive::Vec2D(75.0f, 75.0f));
    artboardInstance->advance(0.0f);
    stateMachineInstance->advanceAndApply(0.0f);

    {
        auto state = stateMachineInstance->layerState(0);
        REQUIRE(state->is<rive::AnimationState>());
        auto animation = state->as<rive::AnimationState>()->animation();
        REQUIRE(animation->name() == "green");
    }
    // Move over the second shape, nothing should change
    stateMachineInstance->pointerMove(rive::Vec2D(200.0f, 75.0f));
    artboardInstance->advance(0.0f);
    stateMachineInstance->advanceAndApply(0.0f);

    {
        auto state = stateMachineInstance->layerState(0);
        REQUIRE(state->is<rive::AnimationState>());
        auto animation = state->as<rive::AnimationState>()->animation();
        REQUIRE(animation->name() == "green");
    }
    // Move out of the second shape, should go back to red
    stateMachineInstance->pointerMove(rive::Vec2D(400.0f, 75.0f));
    artboardInstance->advance(0.0f);
    stateMachineInstance->advanceAndApply(0.0f);

    {
        auto state = stateMachineInstance->layerState(0);
        REQUIRE(state->is<rive::AnimationState>());
        auto animation = state->as<rive::AnimationState>()->animation();
        REQUIRE(animation->name() == "red");
    }
    // Move back into the second shape, should go to green
    stateMachineInstance->pointerMove(rive::Vec2D(200.0f, 75.0f));
    artboardInstance->advance(0.0f);
    stateMachineInstance->advanceAndApply(0.0f);

    {
        auto state = stateMachineInstance->layerState(0);
        REQUIRE(state->is<rive::AnimationState>());
        auto animation = state->as<rive::AnimationState>()->animation();
        REQUIRE(animation->name() == "green");
    }

    delete stateMachineInstance;
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 8 awaits typed Rust execution"]
fn wave_b_hittest_test_008_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Shape clipped by parent layout", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/hit_test_test.riv", &silver);

    auto artboard = file->artboardNamed("ab-1");

    silver.frameSize(artboard->width(), artboard->height());

    REQUIRE(artboard != nullptr);
    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    // Move over the shape
    silver.addFrame();
    stateMachine->pointerMove(rive::Vec2D(50.0f, 150.0f));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    // Move within the shape but the wrapping layout is clipped
    silver.addFrame();
    stateMachine->pointerMove(rive::Vec2D(260.0f, 150.0f));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("hittest_ab1"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 9 awaits typed Rust execution"]
fn wave_b_hittest_test_009_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Shape clipped by parent artboard", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/hit_test_test.riv", &silver);

    auto artboard = file->artboardNamed("ab1-parent");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    // Move over the shape
    silver.addFrame();
    stateMachine->pointerMove(rive::Vec2D(370.0f, 110.0f));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    // Move within the shape but the wrapping parent layout in the nested
    // artboard is clipped
    silver.addFrame();
    stateMachine->pointerMove(rive::Vec2D(370.0f, 180.0f));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("hittest_ab1_parent"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 10 awaits typed Rust execution"]
fn wave_b_hittest_test_010_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Shape clipped by parent and grand-parent artboard", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/hit_test_test.riv", &silver);

    auto artboard = file->artboardNamed("ab1-grand-parent");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    // Move over the shape but outside the grand parent clipping area
    silver.addFrame();
    stateMachine->pointerMove(rive::Vec2D(370.0f, 250.0f));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    // Move within the shape in a non-clipped area
    silver.addFrame();
    stateMachine->pointerMove(rive::Vec2D(370.0f, 190.0f));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    // Move over the shape but outside the parent clipping area
    silver.addFrame();
    stateMachine->pointerMove(rive::Vec2D(510.0f, 190.0f));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("hittest_ab1_grand_parent"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 11 awaits typed Rust execution"]
fn wave_b_hittest_test_011_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Artboard list component with scrolling behavior", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/hit_test_test.riv", &silver);

    auto artboard = file->artboardNamed("ab-2-non-virtualized");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);
    auto scrollProp =
        vmi->propertyValue("scroll-offset")->as<ViewModelInstanceNumber>();

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    silver.addFrame();
    scrollProp->propertyValue(-100);
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    auto initCoord = 0.0f;

    initCoord = 200.0f;
    // First move in an area of the artboard where there are listed components
    // but they are clipped.
    while (initCoord > 100.0f)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(50.0f, initCoord));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
        initCoord -= 10;
    }

    // Next jump to a position of the state to start scrolling the elements
    // Should be noticed that the previous hovered components did not turn green
    initCoord = 75.0f;
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(50.0f, initCoord));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    while (initCoord > -500.0f)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(50.0f, initCoord));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
        initCoord -= 20;
    }
    silver.addFrame();
    stateMachine->pointerUp(rive::Vec2D(50.0f, initCoord));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    // After scroll has ended, move pointer over all visible elements that
    // should turn green
    initCoord = 110.0f;
    while (initCoord > -5.0f)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(50.0f, initCoord));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
        initCoord -= 4;
    }

    CHECK(silver.matches("hittest_ab_2_non_virtualized"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 12 awaits typed Rust execution"]
fn wave_b_hittest_test_012_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE(
    "Artboard list component with scrolling behavior virtualized and carousel",
    "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/hit_test_test.riv", &silver);

    auto artboard = file->artboardNamed("ab-2-virtualized");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);
    auto scrollProp =
        vmi->propertyValue("scroll-offset")->as<ViewModelInstanceNumber>();

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    silver.addFrame();
    scrollProp->propertyValue(-100);
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());

    auto initCoord = 0.0f;

    initCoord = 200.0f;
    // First move in an area of the artboard where there are listed components
    // but they are clipped In this test, since the scroll is virtualized, the
    // pointer is not actually hovering over clipped components at all
    while (initCoord > 100.0f)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(50.0f, initCoord));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
        initCoord -= 10;
    }

    // Next jump to a position of the state to start scrolling the elements
    // Should be noticed that the previous hovered components did not turn green
    initCoord = 75.0f;
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(50.0f, initCoord));
    stateMachine->advanceAndApply(0.1f);
    artboard->draw(renderer.get());
    while (initCoord > -500.0f)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(50.0f, initCoord));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
        initCoord -= 20;
    }
    silver.addFrame();
    stateMachine->pointerUp(rive::Vec2D(50.0f, initCoord));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    // After scroll has ended, move pointer over all visible elements that
    // should turn green
    initCoord = 110.0f;
    while (initCoord > -5.0f)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(50.0f, initCoord));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
        initCoord -= 4;
    }

    CHECK(silver.matches("hittest_ab_2_virtualized"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 13 awaits typed Rust execution"]
fn wave_b_hittest_test_013_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Hit testing text in multiple layouts rotated and scaled", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/hit_test_test.riv", &silver);

    auto artboard = file->artboardNamed("ab-text-parent");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    auto initCoord = 400.0f;
    // First move the cursor from left to right through the text
    // within a clipped and non-clipped area
    while (initCoord < 550.0f)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(initCoord, 320.0f));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
        initCoord += 10;
    }

    initCoord = 200.0f;
    // First move the cursor from top to bottom through the text
    // within a clipped and non-clipped area
    while (initCoord < 450.0f)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(500.0f, initCoord));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
        initCoord += 10;
    }

    CHECK(silver.matches("hittest_ab_text_parent"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 14 awaits typed Rust execution"]
fn wave_b_hittest_test_014_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Hit testing shapes in layouts", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/hit_test_test.riv", &silver);

    auto artboard = file->artboardNamed("ab-shape-parent");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    auto initCoord = 0.0f;
    // First move the cursor from left to right through the text
    // within a clipped and non-clipped area
    while (initCoord < 550.0f)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(310.0f, initCoord));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
        initCoord += 20;
    }

    initCoord = 220.0f;
    // First move the cursor from top to bottom through the text
    // within a clipped and non-clipped area
    while (initCoord < 530.0f)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(initCoord, 420.0f));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
        initCoord += 20;
    }

    CHECK(silver.matches("hittest_ab_shape_parent"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 15 awaits typed Rust execution"]
fn wave_b_hittest_test_015_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Hit testing objects inside shapes", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/hit_test_nested.riv", &silver);

    auto artboard = file->artboardNamed("Main");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    // Hover shape in another shape with no path
    silver.addFrame();
    stateMachine->pointerMove(rive::Vec2D(150.0f, 150.0f));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    // Hover nested artboard in another shape with no path
    silver.addFrame();
    stateMachine->pointerMove(rive::Vec2D(300.0f, 200.0f));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    // Hover text in another shape with path
    silver.addFrame();
    stateMachine->pointerMove(rive::Vec2D(100.0f, 250.0f));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    // Hover nested artboard in another shape with path
    silver.addFrame();
    stateMachine->pointerMove(rive::Vec2D(400.0f, 350.0f));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("hittest_nested"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 16 awaits typed Rust execution"]
fn wave_b_hittest_test_016_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Pointer exit works correctly", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/pointer_exit.riv", &silver);

    auto artboard = file->artboardNamed("main");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());
    // Move from [100.0, 250.0] to [400.0, 250.0]
    // This will hover over two nested artboards and should unhover once an
    // opaque target is hit
    float mousePos = 100.0f;
    for (mousePos = 100.0f; mousePos <= 400; mousePos += 30)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(mousePos, 250.0f));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    // Move from [500.0, 250.0] to [100.0, 250.0]
    // This movement will start at the opaque target and should only trigger the
    // hover effect once it reaches the emousePosed sections of the nseted
    // artboards
    for (mousePos = 500.0f; mousePos > 100; mousePos -= 30)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(mousePos, 250.0f));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    // Move from [240.0, 390.0] to [240.0, 90.0]
    // This movement will start at the opaque target and should only trigger the
    // hover effect once it reaches the emousePosed sections of the nseted
    // artboards
    for (mousePos = 500.0f; mousePos > 100; mousePos -= 30)
    {
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(240.0f, mousePos));
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("pointer_exit"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 17 awaits typed Rust execution"]
fn wave_b_hittest_test_017_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Hit testing multi touch events", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/multitouch.riv", &silver);

    auto artboard = file->artboardNamed("main");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    // Simple click with single pointer
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(200.0f, 350.0f), 1);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->pointerUp(rive::Vec2D(200.0f, 350.0f), 1);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    // New click gesture started with pointer id 1
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(200.0f, 350.0f), 1);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    // Pointer up with pointer id 0 should not complete the click gesture
    silver.addFrame();
    stateMachine->pointerUp(rive::Vec2D(200.0f, 350.0f), 0);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    // Pointer up with pointer id 1 should complete the click gesture
    silver.addFrame();
    stateMachine->pointerUp(rive::Vec2D(200.0f, 350.0f), 1);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    // Two click gestures interleaved: 1 down - 0 down - 0 up - 1 up
    // should toggle color twice
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(200.0f, 350.0f), 1);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(200.0f, 350.0f), 0);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->pointerUp(rive::Vec2D(200.0f, 350.0f), 0);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->pointerUp(rive::Vec2D(200.0f, 350.0f), 1);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("multitouch"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 18 awaits typed Rust execution"]
fn wave_b_hittest_test_018_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Multitouch with nested artboard and pointer exit event", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/multitouch_enter.riv", &silver);

    auto artboard = file->artboardNamed("Main");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    // Script
    // Advancing by 0.0000s
    // Advancing by 0.0167s
    // Touch (id: 9) began at {122.5845,443.8406}
    // Advancing by 0.0167s
    // Touch (id: 8) began at {459.5410,188.4058}
    // Touch (id: 7) began at {333.3333,248.1884}
    // Advancing by 0.0167s
    // Touch (id: 8) ended at {459.5410,188.4058}
    // Touch (id: 8) exited at {459.5410,188.4058}
    // Touch (id: 9) ended at {123.7923,444.4445}
    // Touch (id: 9) exited at {123.7923,444.4445}
    // Touch (id: 7) ended at {333.3333,248.1884}
    // Touch (id: 7) exited at {333.3333,248.1884}
    // Advancing by 0.0167s
    // Touch (id: 7) began at {118.9613,439.6135}
    // Touch (id: 9) began at {346.6183,269.9276}
    // Touch (id: 8) began at {459.5410,194.4444}
    // Advancing by 0.0167s
    // Touch (id: 9) ended at {346.6183,269.9276}
    // Touch (id: 9) exited at {346.6183,269.9276}
    // Touch (id: 7) ended at {122.5845,440.8212}
    // Touch (id: 7) exited at {122.5845,440.8212}
    // Touch (id: 8) ended at {459.5410,194.4444}
    // Touch (id: 8) exited at {459.5410,194.4444}
    // Advancing by 0.0167s

    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(122.5845f, 443.8406f), 9);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(459.5410f, 188.4058f), 8);
    stateMachine->pointerDown(rive::Vec2D(333.3333f, 248.1884f), 7);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerUp(rive::Vec2D(459.5410f, 188.4058f), 8);
    stateMachine->pointerExit(rive::Vec2D(459.5410f, 188.4058f), 8);
    stateMachine->pointerUp(rive::Vec2D(123.7923f, 444.4445f), 9);
    stateMachine->pointerExit(rive::Vec2D(123.7923f, 444.4445f), 9);
    stateMachine->pointerUp(rive::Vec2D(333.3333f, 248.1884f), 7);
    stateMachine->pointerExit(rive::Vec2D(333.3333f, 248.1884f), 7);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(118.9613f, 439.6135f), 7);
    stateMachine->pointerDown(rive::Vec2D(346.6183f, 269.9276f), 9);
    stateMachine->pointerDown(rive::Vec2D(459.5410f, 194.4444f), 8);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerUp(rive::Vec2D(346.6183f, 269.9276f), 9);
    stateMachine->pointerExit(rive::Vec2D(346.6183f, 269.9276f), 9);
    stateMachine->pointerUp(rive::Vec2D(122.5845f, 440.8212f), 7);
    stateMachine->pointerExit(rive::Vec2D(122.5845f, 440.8212f), 7);
    stateMachine->pointerUp(rive::Vec2D(459.5410f, 194.4444f), 8);
    stateMachine->pointerExit(rive::Vec2D(459.5410f, 194.4444f), 8);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("multitouch_enter"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 19 awaits typed Rust execution"]
fn wave_b_hittest_test_019_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Multitouch with list and pointer exit event", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/multitouch_enter.riv", &silver);

    auto artboard = file->artboardNamed("MainList");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    int viewModelId = artboard.get()->viewModelId();

    auto vmi = viewModelId == -1
                   ? file->createViewModelInstance(artboard.get())
                   : file->createViewModelInstance(viewModelId, 0);

    stateMachine->bindViewModelInstance(vmi);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(122.5845f, 443.8406f), 9);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(459.5410f, 188.4058f), 8);
    stateMachine->pointerDown(rive::Vec2D(333.3333f, 248.1884f), 7);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerUp(rive::Vec2D(459.5410f, 188.4058f), 8);
    stateMachine->pointerExit(rive::Vec2D(459.5410f, 188.4058f), 8);
    stateMachine->pointerUp(rive::Vec2D(123.7923f, 444.4445f), 9);
    stateMachine->pointerExit(rive::Vec2D(123.7923f, 444.4445f), 9);
    stateMachine->pointerUp(rive::Vec2D(333.3333f, 248.1884f), 7);
    stateMachine->pointerExit(rive::Vec2D(333.3333f, 248.1884f), 7);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(118.9613f, 439.6135f), 7);
    stateMachine->pointerDown(rive::Vec2D(346.6183f, 269.9276f), 9);
    stateMachine->pointerDown(rive::Vec2D(459.5410f, 194.4444f), 8);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerUp(rive::Vec2D(346.6183f, 269.9276f), 9);
    stateMachine->pointerExit(rive::Vec2D(346.6183f, 269.9276f), 9);
    stateMachine->pointerUp(rive::Vec2D(122.5845f, 440.8212f), 7);
    stateMachine->pointerExit(rive::Vec2D(122.5845f, 440.8212f), 7);
    stateMachine->pointerUp(rive::Vec2D(459.5410f, 194.4444f), 8);
    stateMachine->pointerExit(rive::Vec2D(459.5410f, 194.4444f), 8);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    silver.addFrame();
    stateMachine->pointerMove(rive::Vec2D(50.0f, 300.0f), 0, 7);
    stateMachine->pointerMove(rive::Vec2D(250.0f, 200.0f), 0, 8);
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    float xOffset = 0;
    while (xOffset < 300)
    {
        xOffset += 20;
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(50.0f + xOffset, 300.0f), 0, 7);
        stateMachine->pointerMove(rive::Vec2D(250.0f + xOffset, 200.0f), 0, 8);
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }

    CHECK(silver.matches("multitouch_enter-MainList"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 20 awaits typed Rust execution"]
fn wave_b_hittest_test_020_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Multitouch with multi scroll", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/multitouch_enter.riv", &silver);

    auto artboard = file->artboardNamed("MultiScroll");
    REQUIRE(artboard != nullptr);

    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);
    stateMachine->advanceAndApply(0.1f);

    auto renderer = silver.makeRenderer();
    artboard->draw(renderer.get());

    silver.addFrame();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());
    float yOffset = 400;
    stateMachine->pointerDown(rive::Vec2D(50.0f, yOffset), 7);
    stateMachine->pointerDown(rive::Vec2D(350.0f, yOffset), 8);
    while (yOffset > 0)
    {
        yOffset -= 20;
        silver.addFrame();
        stateMachine->pointerMove(rive::Vec2D(50.0f, yOffset), 0, 7);
        stateMachine->pointerMove(rive::Vec2D(350.0f, yOffset), 0, 8);
        stateMachine->advanceAndApply(0.016f);
        artboard->draw(renderer.get());
    }
    stateMachine->pointerUp(rive::Vec2D(50.0f, yOffset), 7);
    stateMachine->pointerUp(rive::Vec2D(350.0f, yOffset), 8);

    CHECK(silver.matches("multitouch_enter-MultiScroll"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned hittest_test case 21 awaits typed Rust execution"]
fn wave_b_hittest_test_021_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("Hit test leaves in collapsed layouts", "[silver]")
{
    SerializingFactory silver;
    auto file = ReadRiveFile("assets/hittest_collapsed_layouts.riv", &silver);

    auto artboard = file->artboardDefault();
    silver.frameSize(artboard->width(), artboard->height());

    auto stateMachine = artboard->stateMachineAt(0);

    auto vmi = file->createDefaultViewModelInstance(artboard.get());

    stateMachine->bindViewModelInstance(vmi);
    auto renderer = silver.makeRenderer();
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    // Click hides the text successfully
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(250.0f, 50.0f));
    stateMachine->pointerUp(rive::Vec2D(250.0f, 50.0f));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    // Clicking again does not show the text back because hit test doesn't
    // succeed
    silver.addFrame();
    stateMachine->pointerDown(rive::Vec2D(250.0f, 50.0f));
    stateMachine->pointerUp(rive::Vec2D(250.0f, 50.0f));
    stateMachine->advanceAndApply(0.016f);
    artboard->draw(renderer.get());

    CHECK(silver.matches("hittest_collapsed_layouts"));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned ik_constraint_test case 1 awaits typed Rust execution"]
fn wave_b_ik_constraint_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ik with skinned bones orders correctly", "[file]")
{
    auto file = ReadRiveFile("assets/complex_ik_dependency.riv");

    auto artboard = file->artboard();

    REQUIRE(artboard->find<rive::Bone>("One") != nullptr);
    auto one = artboard->find<rive::Bone>("One");

    REQUIRE(artboard->find<rive::Bone>("Two") != nullptr);
    auto two = artboard->find<rive::Bone>("Two");
    rive::Skin* skin = nullptr;
    for (auto object : artboard->objects())
    {
        if (object->is<rive::Skin>())
        {
            skin = object->as<rive::Skin>();
            break;
        }
    }

    REQUIRE(skin != nullptr);
    REQUIRE(two->constraints()[0]->is<rive::IKConstraint>());

    REQUIRE(skin->graphOrder() > one->graphOrder());
    REQUIRE(skin->graphOrder() > two->graphOrder());
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned ik_test case 1 awaits typed Rust execution"]
fn wave_b_ik_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("two bone ik places bones correctly", "[file]")
{
    auto file = ReadRiveFile("assets/two_bone_ik.riv");
    auto artboard = file->artboard();

    REQUIRE(artboard->find<rive::Shape>("circle a") != nullptr);
    auto circleA = artboard->find<rive::Shape>("circle a");

    REQUIRE(artboard->find<rive::Shape>("circle b") != nullptr);
    auto circleB = artboard->find<rive::Shape>("circle b");

    REQUIRE(artboard->find<rive::Bone>("a") != nullptr);
    auto boneA = artboard->find<rive::Bone>("a");

    REQUIRE(artboard->find<rive::Bone>("b") != nullptr);
    auto boneB = artboard->find<rive::Bone>("b");

    REQUIRE(artboard->find<rive::Node>("target") != nullptr);
    auto target = artboard->find<rive::Node>("target");

    REQUIRE(artboard->animation("Animation 1") != nullptr);
    auto animation = artboard->animation("Animation 1");

    // Make sure dependency structure is correct. Important thing here is to
    // ensure that circle a is dependent upon the tip of the ik chain (bone b).
    // circle b is a child of bone b so it'll be there anyway, but may as well
    // validate.
    REQUIRE(std::find(boneB->dependents().begin(),
                      boneB->dependents().end(),
                      circleA) != boneB->dependents().end());
    REQUIRE(std::find(boneB->dependents().begin(),
                      boneB->dependents().end(),
                      circleB) != boneB->dependents().end());

    animation->apply(artboard, 0.0f, 1.0f);
    artboard->advance(0.0f);
    REQUIRE(target->x() == 296.0f);
    REQUIRE(target->y() == 202.0f);
    REQUIRE(aboutEqual(boneA->worldTransform(),
                       rive::Mat2D(0.11632211506366729736328125f,
                                   -0.993211567401885986328125f,
                                   0.993211567401885986328125f,
                                   0.11632211506366729736328125f,
                                   26.015254974365234375f,
                                   475.2149658203125f)));

    REQUIRE(aboutEqual(boneB->worldTransform(),
                       rive::Mat2D(0.974071562290191650390625f,
                                   0.2262403070926666259765625f,
                                   -0.2262403070926666259765625f,
                                   0.974071562290191650390625f,
                                   64.31568145751953125f,
                                   148.1883544921875f)));

    animation->apply(artboard, 1.0f, 1.0f);
    artboard->advance(0.0f);
    REQUIRE(target->x() == 450.0f);
    REQUIRE(target->y() == 337.0f);
    REQUIRE(aboutEqual(boneA->worldTransform(),
                       rive::Mat2D(0.650279819965362548828125f,
                                   -0.7596948146820068359375f,
                                   0.7596948146820068359375f,
                                   0.650279819965362548828125f,
                                   26.015254974365234375f,
                                   475.2149658203125f)));

    REQUIRE(aboutEqual(boneB->worldTransform(),
                       rive::Mat2D(0.8823678493499755859375f,
                                   0.470560371875762939453125f,
                                   -0.47056043148040771484375f,
                                   0.882367908954620361328125f,
                                   240.1275634765625f,
                                   225.07647705078125f)));
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned ik_test case 2 awaits typed Rust execution"]
fn wave_b_ik_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("ik keeps working after a lot of iterations", "[file]")
{
    auto file = ReadRiveFile("assets/two_bone_ik.riv");
    auto artboard = file->artboard();

    REQUIRE(artboard->find<rive::Shape>("circle a") != nullptr);
    auto circleA = artboard->find<rive::Shape>("circle a");

    REQUIRE(artboard->find<rive::Shape>("circle b") != nullptr);
    auto circleB = artboard->find<rive::Shape>("circle b");

    REQUIRE(artboard->find<rive::Bone>("a") != nullptr);
    auto boneA = artboard->find<rive::Bone>("a");

    REQUIRE(artboard->find<rive::Bone>("b") != nullptr);
    auto boneB = artboard->find<rive::Bone>("b");

    REQUIRE(artboard->find<rive::Node>("target") != nullptr);
    auto target = artboard->find<rive::Node>("target");

    REQUIRE(artboard->animation("Animation 1") != nullptr);
    auto animation = artboard->animation("Animation 1");

    // Make sure dependency structure is correct. Important thing here is to
    // ensure that circle a is dependent upon the tip of the ik chain (bone b).
    // circle b is a child of bone b so it'll be there anyway, but may as well
    // validate.
    REQUIRE(std::find(boneB->dependents().begin(),
                      boneB->dependents().end(),
                      circleA) != boneB->dependents().end());
    REQUIRE(std::find(boneB->dependents().begin(),
                      boneB->dependents().end(),
                      circleB) != boneB->dependents().end());

    for (int i = 0; i < 1000; i++)
    {
        animation->apply(artboard, 0.0f, 1.0f);
        artboard->advance(0.0f);
        REQUIRE(target->x() == 296.0f);
        REQUIRE(target->y() == 202.0f);
        REQUIRE(aboutEqual(boneA->worldTransform(),
                           rive::Mat2D(0.11632211506366729736328125f,
                                       -0.993211567401885986328125f,
                                       0.993211567401885986328125f,
                                       0.11632211506366729736328125f,
                                       26.015254974365234375f,
                                       475.2149658203125f)));

        REQUIRE(aboutEqual(boneB->worldTransform(),
                           rive::Mat2D(0.974071562290191650390625f,
                                       0.2262403070926666259765625f,
                                       -0.2262403070926666259765625f,
                                       0.974071562290191650390625f,
                                       64.31568145751953125f,
                                       148.1883544921875f)));

        animation->apply(artboard, 1.0f, 1.0f);
        artboard->advance(0.0f);
        REQUIRE(target->x() == 450.0f);
        REQUIRE(target->y() == 337.0f);
        REQUIRE(aboutEqual(boneA->worldTransform(),
                           rive::Mat2D(0.650279819965362548828125f,
                                       -0.7596948146820068359375f,
                                       0.7596948146820068359375f,
                                       0.650279819965362548828125f,
                                       26.015254974365234375f,
                                       475.2149658203125f)));

        REQUIRE(aboutEqual(boneB->worldTransform(),
                           rive::Mat2D(0.8823678493499755859375f,
                                       0.470560371875762939453125f,
                                       -0.47056043148040771484375f,
                                       0.882367908954620361328125f,
                                       240.1275634765625f,
                                       225.07647705078125f)));
    }
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned image_asset_test case 1 awaits typed Rust execution"]
fn wave_b_image_asset_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("image assets loads correctly", "[assets]")
{
    auto file = ReadRiveFile("assets/walle.riv");

    auto node = file->artboard()->find("walle");
    REQUIRE(node != nullptr);
    REQUIRE(node->is<rive::Image>());
    auto walle = node->as<rive::Image>();
    REQUIRE(walle->imageAsset() != nullptr);
    REQUIRE(walle->imageAsset()->decodedByteSize == 218873);

    auto eve_left = file->artboard()->find("eve_left");
    REQUIRE(eve_left != nullptr);
    REQUIRE(eve_left->is<rive::Image>());
    REQUIRE(eve_left->as<rive::Image>()->imageAsset() != nullptr);
    REQUIRE(eve_left->as<rive::Image>()->imageAsset()->decodedByteSize ==
            246825);

    auto eve_right = file->artboard()->find("eve_right");
    REQUIRE(eve_right != nullptr);
    REQUIRE(eve_right->is<rive::Image>());
    REQUIRE(eve_right->as<rive::Image>()->imageAsset() != nullptr);
    REQUIRE(eve_right->as<rive::Image>()->imageAsset() != walle->imageAsset());
    REQUIRE(eve_right->as<rive::Image>()->imageAsset() ==
            eve_left->as<rive::Image>()->imageAsset());

    file->artboard()->updateComponents();

    rive::NoOpRenderer renderer;
    file->artboard()->draw(&renderer);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned image_asset_test case 2 awaits typed Rust execution"]
fn wave_b_image_asset_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("out of band image assets loads correctly", "[assets]")
{
    rive::NoOpFactory gEmptyFactory;

    std::string filename = "assets/out_of_band/walle.riv";
    rive::RelativeLocalAssetLoader loader(filename);

    auto file = ReadRiveFile(filename.c_str(), &gEmptyFactory, &loader);

    auto node = file->artboard()->find("walle");
    REQUIRE(node != nullptr);
    REQUIRE(node->is<rive::Image>());
    auto walle = node->as<rive::Image>();
    REQUIRE(walle->imageAsset() != nullptr);
    REQUIRE(walle->imageAsset()->decodedByteSize == 218873);

    auto eve_left = file->artboard()->find("eve_left");
    REQUIRE(eve_left != nullptr);
    REQUIRE(eve_left->is<rive::Image>());
    REQUIRE(eve_left->as<rive::Image>()->imageAsset() != nullptr);
    REQUIRE(eve_left->as<rive::Image>()->imageAsset()->decodedByteSize ==
            246825);

    auto eve_right = file->artboard()->find("eve_right");
    REQUIRE(eve_right != nullptr);
    REQUIRE(eve_right->is<rive::Image>());
    REQUIRE(eve_right->as<rive::Image>()->imageAsset() != nullptr);
    REQUIRE(eve_right->as<rive::Image>()->imageAsset() != walle->imageAsset());
    REQUIRE(eve_right->as<rive::Image>()->imageAsset() ==
            eve_left->as<rive::Image>()->imageAsset());

    file->artboard()->updateComponents();

    rive::NoOpRenderer renderer;
    file->artboard()->draw(&renderer);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned image_decoders_test case 1 awaits typed Rust execution"]
fn wave_b_image_decoders_test_001_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("png file decodes correctly", "[image-decoder]")
{
    auto file = ReadFile("assets/placeholder.png");
    REQUIRE(file.size() == 1096);

    auto bitmap = Bitmap::decode(file.data(), file.size());

    REQUIRE(bitmap != nullptr);

    REQUIRE(bitmap->width() == 226);
    REQUIRE(bitmap->height() == 128);
    const size_t channels =
        bitmap->pixelFormat() == Bitmap::PixelFormat::RGB ? 3 : 4;
    REQUIRE(bitmap->numBytes() == 226 * 128 * channels);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned image_decoders_test case 2 awaits typed Rust execution"]
fn wave_b_image_decoders_test_002_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("jpeg file decodes correctly", "[image-decoder]")
{
    auto file = ReadFile("assets/open_source.jpg");
    REQUIRE(file.size() == 8880);

    auto bitmap = Bitmap::decode(file.data(), file.size());

    REQUIRE(bitmap != nullptr);

    REQUIRE(bitmap->width() == 350);
    REQUIRE(bitmap->height() == 200);
    const size_t channels =
        bitmap->pixelFormat() == Bitmap::PixelFormat::RGB ? 3 : 4;
    REQUIRE(bitmap->numBytes() == 350 * 200 * channels);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned image_decoders_test case 3 awaits typed Rust execution"]
fn wave_b_image_decoders_test_003_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("bad jpeg file doesn't cause an overflow", "[image-decoder]")
{
    auto file = ReadFile("assets/bad.jpg");
    REQUIRE(file.size() == 88731);

    auto bitmap = Bitmap::decode(file.data(), file.size());

    REQUIRE(bitmap != nullptr);

    REQUIRE(bitmap->width() == 24566);
    REQUIRE(bitmap->height() == 58278);
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned image_decoders_test case 4 awaits typed Rust execution"]
fn wave_b_image_decoders_test_004_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("bad png file doesn't cause an overflow", "[image-decoder]")
{
    auto file = ReadFile("assets/bad.png");
    REQUIRE(file.size() == 534283);

    auto bitmap = Bitmap::decode(file.data(), file.size());

#ifdef __APPLE__
    // Loading this bad PNG file in CG actually works and we do get an image
    // albiet black
    REQUIRE(bitmap != nullptr);

    REQUIRE(bitmap->width() == 58278);
    REQUIRE(bitmap->height() == 24566);
#else
    // Our decoders return null as we have an invalid header with bogus
    // resolution and we want to avoid a potential attack vector
    REQUIRE(bitmap == nullptr);
#endif
}"########,
    );
}

#[test]
#[ignore = "expected-red: complete pinned image_decoders_test case 5 awaits typed Rust execution"]
fn wave_b_image_decoders_test_005_direct_port_expected_red() {
    pending_literal_port(
        r########"TEST_CASE("webp file decodes correctly", "[image-decoder]")
{
    auto file = ReadFile("assets/1.webp");
    REQUIRE(file.size() == 30320);

    auto bitmap = Bitmap::decode(file.data(), file.size());

    REQUIRE(bitmap != nullptr);

    REQUIRE(bitmap->width() == 550);
    REQUIRE(bitmap->height() == 368);
    REQUIRE(bitmap->numBytes() == 550 * 368 * 4);
}"########,
    );
}
