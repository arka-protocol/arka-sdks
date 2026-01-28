package com.arka.sdk.plugin;

import com.arka.sdk.types.PactEvent;
import com.arka.sdk.types.PactRule;
import java.util.List;
import java.util.function.Consumer;
import java.util.function.Function;

/**
 * Plugin lifecycle hooks.
 */
public class PluginHooks {
    private Runnable onLoad;
    private Runnable onUnload;
    private Function<PactEvent, PactEvent> beforeEventProcess;
    private Consumer<PactEvent> afterDecision;
    private Consumer<List<PactRule>> onRulesUpdated;

    public PluginHooks onLoad(Runnable handler) {
        this.onLoad = handler;
        return this;
    }

    public PluginHooks onUnload(Runnable handler) {
        this.onUnload = handler;
        return this;
    }

    public PluginHooks beforeEventProcess(Function<PactEvent, PactEvent> handler) {
        this.beforeEventProcess = handler;
        return this;
    }

    public PluginHooks afterDecision(Consumer<PactEvent> handler) {
        this.afterDecision = handler;
        return this;
    }

    public PluginHooks onRulesUpdated(Consumer<List<PactRule>> handler) {
        this.onRulesUpdated = handler;
        return this;
    }

    public void triggerOnLoad() {
        if (onLoad != null) onLoad.run();
    }

    public void triggerOnUnload() {
        if (onUnload != null) onUnload.run();
    }

    public PactEvent triggerBeforeEventProcess(PactEvent event) {
        if (beforeEventProcess != null) return beforeEventProcess.apply(event);
        return event;
    }

    public void triggerAfterDecision(PactEvent event) {
        if (afterDecision != null) afterDecision.accept(event);
    }

    public void triggerOnRulesUpdated(List<PactRule> rules) {
        if (onRulesUpdated != null) onRulesUpdated.accept(rules);
    }
}
