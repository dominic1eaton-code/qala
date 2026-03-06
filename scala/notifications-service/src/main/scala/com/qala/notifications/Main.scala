package com.qala.notifications

import com.sun.net.httpserver.{HttpExchange, HttpHandler, HttpServer}
import java.net.InetSocketAddress
import java.nio.charset.StandardCharsets
import java.time.Instant
import java.util.concurrent.ConcurrentHashMap
import scala.jdk.CollectionConverters._

object Main {
  final case class Notification(id: String, userId: String, channel: String, message: String, createdAt: String)

  private val notifications = new ConcurrentHashMap[String, Notification]()
  private val events = new java.util.concurrent.CopyOnWriteArrayList[String]()

  def main(args: Array[String]): Unit = {
    val port = sys.env.get("PORT").flatMap(v => util.Try(v.toInt).toOption).getOrElse(8089)
    val server = HttpServer.create(new InetSocketAddress(port), 0)
    server.createContext("/health", jsonGet("""{"service":"notifications-service","status":"ok"}"""))
    server.createContext("/notify", new NotifyHandler)
    server.createContext("/notifications", new NotificationHandler)
    server.createContext("/events", new EventsHandler)
    server.setExecutor(null)
    println(s"[notifications-service] listening on :$port")
    server.start()
  }

  private class NotifyHandler extends HttpHandler {
    override def handle(exchange: HttpExchange): Unit = {
      if (exchange.getRequestMethod != "POST") {
        write(exchange, 405, """{"error":"method not allowed"}""")
        return
      }
      val body = readBody(exchange)
      val userId = extractJsonField(body, "user_id").getOrElse("unknown")
      val channel = extractJsonField(body, "channel").getOrElse("in-app")
      val message = extractJsonField(body, "message").getOrElse("notification")
      val id = s"ntf-${System.currentTimeMillis()}"
      val notification = Notification(id, userId, channel, message, Instant.now().toString)
      notifications.put(id, notification)
      events.add(s"""{"topic":"NOTIFICATIONS","type":"NOTIFICATION_SENT","notification_id":"$id"}""")
      write(exchange, 200, s"""{"status":"sent","notification_id":"$id"}""")
    }
  }

  private class NotificationHandler extends HttpHandler {
    override def handle(exchange: HttpExchange): Unit = {
      if (exchange.getRequestMethod != "GET") {
        write(exchange, 405, """{"error":"method not allowed"}""")
        return
      }
      val path = exchange.getRequestURI.getPath
      val parts = path.split("/").filter(_.nonEmpty)
      if (parts.length >= 2 && parts(0) == "notifications") {
        val id = parts(1)
        Option(notifications.get(id)) match {
          case Some(item) =>
            write(exchange, 200, s"""{"id":"${item.id}","user_id":"${item.userId}","channel":"${item.channel}","message":"${escape(item.message)}","created_at":"${item.createdAt}"}""")
          case None =>
            write(exchange, 404, """{"error":"notification not found"}""")
        }
      } else {
        val items = notifications.values().asScala
          .map(n => s"""{"id":"${n.id}","user_id":"${n.userId}","channel":"${n.channel}","message":"${escape(n.message)}","created_at":"${n.createdAt}"}""")
          .mkString("[", ",", "]")
        write(exchange, 200, s"""{"count":${notifications.size()},"items":$items}""")
      }
    }
  }

  private class EventsHandler extends HttpHandler {
    override def handle(exchange: HttpExchange): Unit = {
      if (exchange.getRequestMethod != "GET") {
        write(exchange, 405, """{"error":"method not allowed"}""")
        return
      }
      write(exchange, 200, s"""{"count":${events.size()},"items":[${events.asScala.mkString(",")}]}""")
    }
  }

  private def jsonGet(payload: String): HttpHandler = new HttpHandler {
    override def handle(exchange: HttpExchange): Unit = {
      if (exchange.getRequestMethod != "GET") {
        write(exchange, 405, """{"error":"method not allowed"}""")
        return
      }
      write(exchange, 200, payload)
    }
  }

  private def readBody(exchange: HttpExchange): String = {
    new String(exchange.getRequestBody.readAllBytes(), StandardCharsets.UTF_8)
  }

  private def write(exchange: HttpExchange, status: Int, payload: String): Unit = {
    val bytes = payload.getBytes(StandardCharsets.UTF_8)
    exchange.getResponseHeaders.add("Content-Type", "application/json")
    exchange.sendResponseHeaders(status, bytes.length)
    val os = exchange.getResponseBody
    os.write(bytes)
    os.close()
  }

  private def extractJsonField(body: String, key: String): Option[String] = {
    val pattern = ("\"" + key + "\"\\s*:\\s*\"([^\"]+)\"").r
    pattern.findFirstMatchIn(body).map(_.group(1))
  }

  private def escape(value: String): String = value.replace("\"", "\\\"")
}
