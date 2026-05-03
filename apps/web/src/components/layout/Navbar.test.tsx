import { describe, expect, test, beforeEach } from "bun:test";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

import { Navbar } from "./Navbar";

describe("Navbar", () => {
  beforeEach(() => {
    render(<Navbar />);
  });

  test("renders navigation links", () => {
    expect(screen.getByRole("link", { name: "Gunmetal Home" })).toBeDefined();
    expect(screen.getByRole("button", { name: "Products", expanded: false })).toBeDefined();
    expect(screen.getByRole("button", { name: "Developer", expanded: false })).toBeDefined();
    expect(screen.getByRole("button", { name: "Resources", expanded: false })).toBeDefined();
    expect(screen.getAllByRole("link", { name: "Download" }).length).toBeGreaterThanOrEqual(1);
    expect(screen.getAllByRole("link", { name: "GitHub" }).length).toBeGreaterThanOrEqual(1);
  });

  test("dropdown keyboard navigation opens with Enter and closes with Escape", () => {
    const productsBtn = screen.getByRole("button", { name: "Products", expanded: false });

    fireEvent.keyDown(productsBtn, { key: "Enter", code: "Enter" });
    expect(productsBtn.getAttribute("aria-expanded")).toBe("true");

    const menu = screen.getByRole("menu", { name: "Products" });
    expect(menu.hidden).toBe(false);
    expect(screen.getByRole("menuitem", { name: /Gunmetal/ })).toBeDefined();

    fireEvent.keyDown(productsBtn, { key: "Escape", code: "Escape" });
    expect(productsBtn.getAttribute("aria-expanded")).toBe("false");
  });

  test("dropdown keyboard navigation opens with Space", () => {
    const productsBtn = screen.getByRole("button", { name: "Products", expanded: false });

    fireEvent.keyDown(productsBtn, { key: " ", code: "Space" });
    expect(productsBtn.getAttribute("aria-expanded")).toBe("true");

    const menu = screen.getByRole("menu", { name: "Products" });
    expect(menu.hidden).toBe(false);
  });

  test("dropdown keyboard navigation with ArrowDown focuses first menu item", () => {
    const productsBtn = screen.getByRole("button", { name: "Products", expanded: false });

    fireEvent.keyDown(productsBtn, { key: "ArrowDown", code: "ArrowDown" });
    expect(productsBtn.getAttribute("aria-expanded")).toBe("true");

    const firstLink = screen.getByRole("menuitem", { name: /Gunmetal/ });
    expect(document.activeElement).toBe(firstLink);
  });

  test("mobile accordion toggle expands and collapses", async () => {
    const menuBtn = screen.getByRole("button", { name: "Menu" });

    fireEvent.click(menuBtn);
    expect(menuBtn.getAttribute("aria-expanded")).toBe("true");

    const productsAccordion = screen.getAllByRole("button", { name: "Products" })[1];
    fireEvent.click(productsAccordion);

    await waitFor(() => {
      expect(screen.getAllByText("Gunmetal").length).toBeGreaterThanOrEqual(2);
    });

    fireEvent.click(productsAccordion);
  });
});
