use crate::components::*;
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let modal_open = RwSignal::new(false);
    let input_value = RwSignal::new(String::new());
    let textarea_value = RwSignal::new(String::new());
    let checkbox_checked = RwSignal::new(true);
    let toggle_checked = RwSignal::new(true);
    let radio_value = RwSignal::new("standard".to_string());
    let dropdown_selected = RwSignal::new("".to_string());
    let active_tab = RwSignal::new("description".to_string());
    let active_step = RwSignal::new(1usize);
    let current_page = RwSignal::new(1usize);
    let search_value = RwSignal::new(String::new());
    let progress_value = RwSignal::new(25u32);
    let profile_progress = RwSignal::new(60u32);
    let upload_progress = RwSignal::new(90u32);
    let cart_count = RwSignal::new(3u32);

    let dropdown_options = vec![
        DropdownItem {
            value: "50ml".to_string(),
            label: "50ml — $195".to_string(),
        },
        DropdownItem {
            value: "100ml".to_string(),
            label: "100ml — $320".to_string(),
        },
        DropdownItem {
            value: "200ml".to_string(),
            label: "200ml — $450".to_string(),
        },
    ];

    let tabs_data = vec![
        TabItem {
            id: "description".to_string(),
            label: "Description".to_string(),
        },
        TabItem {
            id: "ingredients".to_string(),
            label: "Ingredients".to_string(),
        },
        TabItem {
            id: "reviews".to_string(),
            label: "Reviews".to_string(),
        },
    ];

    let stepper_steps = vec![
        StepperStep {
            number: 1,
            label: "Cart".to_string(),
        },
        StepperStep {
            number: 2,
            label: "Shipping".to_string(),
        },
        StepperStep {
            number: 3,
            label: "Payment".to_string(),
        },
        StepperStep {
            number: 4,
            label: "Confirm".to_string(),
        },
    ];

    let breadcrumbs_items = vec![
        BreadcrumbItem {
            label: "Home".to_string(),
            href: Some("#".to_string()),
        },
        BreadcrumbItem {
            label: "Collection".to_string(),
            href: Some("#".to_string()),
        },
        BreadcrumbItem {
            label: "Signature Series".to_string(),
            href: None,
        },
    ];

    let table_headers = vec![
        TableHeader {
            label: "Reference".to_string(),
        },
        TableHeader {
            label: "Product".to_string(),
        },
        TableHeader {
            label: "Size".to_string(),
        },
        TableHeader {
            label: "Price".to_string(),
        },
        TableHeader {
            label: "Status".to_string(),
        },
    ];

    let table_rows = vec![
        TableRow {
            id: "016-A".to_string(),
            cells: vec![
                "016-A".to_string(),
                "Santal 33".to_string(),
                "100ml".to_string(),
                "$320.00".to_string(),
                "In Stock".to_string(),
            ],
        },
        TableRow {
            id: "033-B".to_string(),
            cells: vec![
                "033-B".to_string(),
                "Another 13".to_string(),
                "50ml".to_string(),
                "$195.00".to_string(),
                "In Stock".to_string(),
            ],
        },
    ];

    let nav_items = vec![
        NavItem {
            label: "Collection".to_string(),
            href: "#".to_string(),
        },
        NavItem {
            label: "About".to_string(),
            href: "#".to_string(),
        },
        NavItem {
            label: "Journal".to_string(),
            href: "#".to_string(),
        },
    ];

    let footer_sections = vec![
        FooterSection {
            title: "Collection".to_string(),
            links: vec![
                FooterLink {
                    label: "All Products".to_string(),
                    href: "#".to_string(),
                },
                FooterLink {
                    label: "New Arrivals".to_string(),
                    href: "#".to_string(),
                },
            ],
        },
        FooterSection {
            title: "Company".to_string(),
            links: vec![
                FooterLink {
                    label: "About Us".to_string(),
                    href: "#".to_string(),
                },
                FooterLink {
                    label: "Contact".to_string(),
                    href: "#".to_string(),
                },
            ],
        },
    ];

    view! {
        <body class="min-h-screen paper-texture">
            <Navbar
                brand="Le Style"
                items=nav_items
                cart_count=cart_count
            />

            <main class="max-w-6xl mx-auto px-6 py-16 relative z-10">
                <header class="mb-24">
                    <div class="flex items-center gap-4 mb-8">
                        <Divider class="flex-1" />
                        <span class="font-mono text-xs tracking-widest text-[var(--fg-muted)]">"COMPLETE UI SYSTEM"</span>
                        <Divider class="flex-1" />
                    </div>

                    <h1 class="font-serif text-6xl md:text-8xl font-light tracking-tight text-center mb-4">
                        "Golden Set"
                    </h1>

                    <p class="font-mono text-xs tracking-widest text-center text-[var(--fg-muted)] uppercase max-w-xl mx-auto">
                        "A comprehensive collection of interface components"
                    </p>
                </header>

                <section class="mb-20">
                    <div class="flex items-center gap-4 mb-8">
                        <span class="number-badge">01</span>
                        <h2 class="font-serif text-3xl font-light">"Navigation"</h2>
                        <Divider class="flex-1" />
                    </div>

                    <div class="grid gap-8">
                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-4">"Breadcrumbs"</span>
                            <Breadcrumbs items=breadcrumbs_items />
                        </Card>

                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-4">"Tabs"</span>
                            <Tabs
                                tabs=tabs_data
                                active=active_tab
                            />
                            <div class="font-mono text-sm text-[var(--fg-muted)] mt-4">
                                {move || match active_tab.get().as_str() {
                                    "description" => "A carefully composed unisex fragrance with woody notes.",
                                    "ingredients" => "Sandalwood, cedarwood, cardamom, iris, violet, amber.",
                                    "reviews" => "4.9 stars from 847 reviews.",
                                    _ => "",
                                }}
                            </div>
                        </Card>

                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Stepper"</span>
                            <Stepper steps=stepper_steps active=active_step />
                        </Card>

                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-4">"Pagination"</span>
                            <div class="flex justify-between items-center">
                                <Pagination
                                    current_page=current_page
                                    total_pages=12
                                />
                                <span class="font-mono text-xs text-[var(--fg-muted)]">"Showing 1-12 of 144 items"</span>
                            </div>
                        </Card>
                    </div>
                </section>

                <section class="mb-20">
                    <div class="flex items-center gap-4 mb-8">
                        <span class="number-badge">02</span>
                        <h2 class="font-serif text-3xl font-light">"Form Controls"</h2>
                        <Divider class="flex-1" />
                    </div>

                    <div class="grid md:grid-cols-2 gap-8">
                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Input Fields"</span>
                            <div class="space-y-4">
                                <div>
                                    <label class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)] uppercase block mb-2">"Default"</label>
                                    <Input placeholder="Enter text" value=input_value />
                                </div>
                                <div>
                                    <label class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)] uppercase block mb-2">"Textarea"</label>
                                    <Input
                                        placeholder="Additional notes..."
                                        rows=3
                                        value=textarea_value
                                    />
                                </div>
                            </div>
                        </Card>

                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Search & Dropdown"</span>
                            <Search
                                placeholder="Search products..."
                                value=search_value
                                class="mb-6"
                            />
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-4">"Dropdown"</span>
                            <Dropdown
                                options=dropdown_options
                                selected=dropdown_selected
                                placeholder="Select Size"
                            />
                        </Card>

                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Checkboxes"</span>
                            <div class="space-y-4">
                                <Checkbox
                                    checked=checkbox_checked
                                    label="Include gift wrapping"
                                />
                                <Checkbox
                                    checked=Memo::new(move |_| false)
                                    label="Express processing (+$15)"
                                />
                            </div>
                        </Card>

                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Radio & Toggle"</span>
                            <div class="space-y-4 mb-8">
                                <Radio
                                    name="delivery"
                                    value="standard"
                                    checked=Memo::new(move |_| radio_value.get() == "standard")
                                    label="Standard Delivery (3-5 days)"
                                />
                                <Radio
                                    name="delivery"
                                    value="express"
                                    checked=Memo::new(move |_| radio_value.get() == "express")
                                    label="Express Delivery (1-2 days)"
                                />
                            </div>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-4">"Toggles"</span>
                            <div class="space-y-4">
                                <Toggle
                                    checked=toggle_checked
                                    label="Email notifications"
                                />
                                <Toggle
                                    checked=Memo::new(move |_| false)
                                    label="SMS updates"
                                />
                            </div>
                        </Card>
                    </div>
                </section>

                <section class="mb-20">
                    <div class="flex items-center gap-4 mb-8">
                        <span class="number-badge">03</span>
                        <h2 class="font-serif text-3xl font-light">"Buttons"</h2>
                        <Divider class="flex-1" />
                    </div>

                    <div class="grid gap-8">
                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Button Variants"</span>
                            <div class="flex flex-wrap gap-4 items-end">
                                <Button>"Default"</Button>
                                <Button variant=ButtonVariant::Filled>"Filled"</Button>
                                <Button variant=ButtonVariant::Olive>"Olive"</Button>
                                <Button variant=ButtonVariant::Ghost>"Ghost"</Button>
                            </div>
                        </Card>

                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Button Sizes"</span>
                            <div class="flex flex-wrap gap-4 items-center">
                                <Button size=ButtonSize::Small>"Small"</Button>
                                <Button>"Default"</Button>
                                <Button size=ButtonSize::Large>"Large"</Button>
                            </div>
                        </Card>

                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Button States"</span>
                            <div class="flex flex-wrap gap-4 items-center">
                                <Button disabled=true>"Disabled"</Button>
                                <Button variant=ButtonVariant::Filled>"With Icon"</Button>
                                <Button>"Dashed Border"</Button>
                            </div>
                        </Card>
                    </div>
                </section>

                <section class="mb-20">
                    <div class="flex items-center gap-4 mb-8">
                        <span class="number-badge">04</span>
                        <h2 class="font-serif text-3xl font-light">"Cards & Containers"</h2>
                        <Divider class="flex-1" />
                    </div>

                    <div class="grid md:grid-cols-3 gap-6 mb-8">
                        <Card>
                            <span class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)] uppercase">"Basic Card"</span>
                            <h3 class="font-serif text-xl mt-2 mb-2">"Simple Container"</h3>
                            <p class="font-mono text-xs text-[var(--fg-muted)] leading-relaxed">
                                "A minimal card with clean borders."
                            </p>
                        </Card>

                        <Card shadow=true>
                            <span class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)] uppercase">"Shadow Card"</span>
                            <h3 class="font-serif text-xl mt-2 mb-2">"With Depth"</h3>
                            <p class="font-mono text-xs text-[var(--fg-muted)] leading-relaxed">
                                "Adds visual weight with an offset shadow effect."
                            </p>
                        </Card>

                        <LabelFrame>
                            <span class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)] uppercase">"Label Frame"</span>
                            <h3 class="font-serif text-xl mt-2 mb-2">"Double Border"</h3>
                            <p class="font-mono text-xs text-[var(--fg-muted)] leading-relaxed">
                                "An elegant double border treatment."
                            </p>
                        </LabelFrame>
                    </div>

                    <Card class="mb-8">
                        <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Tags & Badges"</span>
                        <div class="flex flex-wrap gap-2 mb-4">
                            <Tag>"Default"</Tag>
                            <Tag variant=TagVariant::Filled>"Filled"</Tag>
                            <Tag variant=TagVariant::Olive>"Olive"</Tag>
                            <Tag variant=TagVariant::Terracotta>"Terracotta"</Tag>
                        </div>
                        <div class="flex items-center gap-4">
                            <Button class="btn-sm relative">
                                "Cart"
                                <Badge class="absolute -top-2 -right-2">3</Badge>
                            </Button>
                            <Button class="btn-sm relative">
                                "Notifications"
                                <Badge class="absolute -top-2 -right-2 !bg-[var(--accent-terracotta)] !text-white !border-[var(--accent-terracotta)]">12</Badge>
                            </Button>
                        </div>
                    </Card>

                    <Card>
                        <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Avatars"</span>
                        <div class="flex items-end gap-8">
                            <div class="text-center">
                                <Avatar initials="JD" size=AvatarSize::Small />
                                <span class="font-mono text-[9px] text-[var(--fg-muted)]">"Small"</span>
                            </div>
                            <div class="text-center">
                                <Avatar initials="AB" />
                                <span class="font-mono text-[9px] text-[var(--fg-muted)]">"Default"</span>
                            </div>
                            <div class="text-center">
                                <Avatar initials="MK" size=AvatarSize::Large />
                                <span class="font-mono text-[9px] text-[var(--fg-muted)]">"Large"</span>
                            </div>
                        </div>

                        <div class="mt-8">
                            <span class="font-mono text-[9px] tracking-widest text-[var(--fg-muted)] uppercase block mb-4">"Avatar Group"</span>
                            <AvatarGroup>
                                <Avatar initials="JD" />
                                <Avatar initials="AB" />
                                <Avatar initials="MK" />
                                <Avatar initials="+" />
                            </AvatarGroup>
                        </div>
                    </Card>
                </section>

                <section class="mb-20">
                    <div class="flex items-center gap-4 mb-8">
                        <span class="number-badge">05</span>
                        <h2 class="font-serif text-3xl font-light">"Feedback"</h2>
                        <Divider class="flex-1" />
                    </div>

                    <div class="grid gap-6 mb-8">
                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Alerts"</span>
                            <div class="space-y-4">
                                <Alert
                                    alert_type=AlertType::Success
                                    title="Order Confirmed"
                                    message="Your order has been successfully placed."
                                />
                                <Alert
                                    alert_type=AlertType::Warning
                                    title="Limited Stock"
                                    message="Only 3 items remaining in stock."
                                />
                                <Alert
                                    alert_type=AlertType::Error
                                    title="Payment Failed"
                                    message="Please check your payment details."
                                />
                            </div>
                        </Card>

                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Progress"</span>
                            <div class="space-y-6">
                                <ProgressBar
                                    value=progress_value
                                    max=100
                                    label="CHECKOUT PROGRESS"
                                />
                                <ProgressBar
                                    value=profile_progress
                                    max=100
                                    label="PROFILE COMPLETION"
                                />
                                <ProgressBar
                                    value=upload_progress
                                    max=100
                                    label="UPLOAD STATUS"
                                />
                            </div>
                        </Card>

                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Tooltips"</span>
                            <div class="flex gap-8 justify-center">
                                <Tooltip text="Action description">
                                    <Button size=ButtonSize::Small>"Hover Me"</Button>
                                </Tooltip>
                                <Tooltip text="Click to proceed">
                                    <Button variant=ButtonVariant::Filled size=ButtonSize::Small>"Information"</Button>
                                </Tooltip>
                            </div>
                        </Card>
                    </div>
                </section>

                <section class="mb-20">
                    <div class="flex items-center gap-4 mb-8">
                        <span class="number-badge">06</span>
                        <h2 class="font-serif text-3xl font-light">"Data Display"</h2>
                        <Divider class="flex-1" variant=DividerVariant::Double />
                    </div>

                    <Card class="mb-8">
                        <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Table"</span>
                        <Table headers=table_headers rows=table_rows />
                    </Card>

                    <Card>
                        <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Loading Skeleton"</span>
                        <div class="space-y-4">
                            <div class="flex gap-4 items-center">
                                <Skeleton width="48px" height="48px" class="!border border-[var(--border-light)]" />
                                <div class="flex-1 space-y-2">
                                    <Skeleton width="75%" height="16px" class="!border border-[var(--border-light)]" />
                                    <Skeleton width="50%" height="12px" class="!border border-[var(--border-light)]" />
                                </div>
                            </div>
                            <Skeleton height="80px" class="!border border-[var(--border-light)]" />
                        </div>
                    </Card>
                </section>

                <section class="mb-20">
                    <div class="flex items-center gap-4 mb-8">
                        <span class="number-badge">07</span>
                        <h2 class="font-serif text-3xl font-light">"Overlays"</h2>
                        <Divider class="flex-1" />
                    </div>

                    <div class="grid md:grid-cols-2 gap-8">
                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Modal Dialog"</span>
                            <Button variant=ButtonVariant::Filled on_click=Callback::new(move |_| modal_open.set(true))>
                                "Open Modal"
                            </Button>
                        </Card>

                        <Card>
                            <span class="font-mono text-[10px] tracking-widest text-[var(--fg-muted)] uppercase block mb-6">"Accordion"</span>
                            <AccordionItem header="SHIPPING INFORMATION">
                                "We offer complimentary standard shipping on all orders over $150."
                            </AccordionItem>
                            <AccordionItem header="RETURN POLICY">
                                "Returns accepted within 30 days of purchase."
                            </AccordionItem>
                            <AccordionItem header="INGREDIENTS">
                                "Sandalwood, cedarwood, cardamom, iris, violet, amber."
                            </AccordionItem>
                        </Card>
                    </div>
                </section>

                <section class="mb-20">
                    <div class="flex items-center gap-4 mb-8">
                        <span class="number-badge">08</span>
                        <h2 class="font-serif text-3xl font-light">"Labels & Stamps"</h2>
                        <Divider class="flex-1" />
                    </div>

                    <div class="grid md:grid-cols-3 gap-6">
                        <Card class="flex items-center justify-center bg-[var(--bg-aged)]">
                            <Stamp text="Limited Edition" />
                        </Card>
                    </div>
                </section>
            </main>

            <Footer
                brand="Le Style"
                sections=footer_sections
                description="A complete design system inspired by artisan craftsmanship and timeless aesthetics."
                newsletter_placeholder="Your email"
            />

            <Modal
                is_open=modal_open
                on_close=Callback::new(move |_| modal_open.set(false))
                title="Confirm Action"
            >
                <p class="font-mono text-sm text-[var(--fg-muted)] mb-6">
                    "Are you sure you want to proceed with this action?"
                </p>
                <Divider class="mb-6" />
                <div class="flex gap-4 justify-end">
                    <Button variant=ButtonVariant::Ghost on_click=Callback::new(move |_| modal_open.set(false))>
                        "Cancel"
                    </Button>
                    <Button variant=ButtonVariant::Filled on_click=Callback::new(move |_| modal_open.set(false))>
                        "Confirm"
                    </Button>
                </div>
            </Modal>
        </body>
    }
}
