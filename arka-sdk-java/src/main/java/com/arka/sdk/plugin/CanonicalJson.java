package com.arka.sdk.plugin;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.fasterxml.jackson.datatype.jsr310.JavaTimeModule;

import java.nio.charset.StandardCharsets;

/**
 * Canonical JSON serialization with sorted keys.
 */
public class CanonicalJson {
    private static final ObjectMapper MAPPER = new ObjectMapper()
        .registerModule(new JavaTimeModule())
        .configure(SerializationFeature.ORDER_MAP_ENTRIES_BY_KEYS, true)
        .configure(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS, false);

    public static byte[] serialize(Object data) {
        try {
            String json = MAPPER.writeValueAsString(data);
            return json.getBytes(StandardCharsets.UTF_8);
        } catch (Exception e) {
            throw new RuntimeException("Failed to serialize data", e);
        }
    }

    public static Object deserialize(byte[] data) {
        try {
            String json = new String(data, StandardCharsets.UTF_8);
            return MAPPER.readValue(json, Object.class);
        } catch (Exception e) {
            throw new RuntimeException("Failed to deserialize data", e);
        }
    }

    public static String toJson(Object data) {
        try {
            return MAPPER.writeValueAsString(data);
        } catch (Exception e) {
            throw new RuntimeException("Failed to serialize data", e);
        }
    }

    public static <T> T fromJson(String json, Class<T> type) {
        try {
            return MAPPER.readValue(json, type);
        } catch (Exception e) {
            throw new RuntimeException("Failed to deserialize data", e);
        }
    }
}
